use crate::domain::orders::{CreateOrderWithItemsParams, OrderRepo};
use crate::models;
use serde_json::Value;
use sqlx::{MySql, Pool, QueryBuilder};
use std::sync::Arc;

pub struct OrderRepository {
    pool: Pool<MySql>,
}

impl OrderRepository {
    pub fn new(pool: Pool<MySql>) -> Arc<Self> {
        Arc::new(Self { pool })
    }

    async fn create_order_with_items_internal(
        &self,
        params: CreateOrderWithItemsParams,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let CreateOrderWithItemsParams {
            order_id,
            user_id,
            total_amount,
            pay_amount,
            order_status,
            consignee_info,
            items,
        } = params;

        sqlx::query(
            r#"INSERT INTO wx_orders (order_id, user_id, total_amount, pay_amount, order_status, consignee_info)
               VALUES (?, ?, ?, ?, ?, ?)"#,
        )
            .bind(&order_id)
            .bind(user_id)
            .bind(total_amount)
            .bind(pay_amount)
            .bind(order_status)
            .bind(consignee_info)
            .execute(&mut *tx)
            .await?;

        let mut sorted_items = items;
        sorted_items.sort_by_key(|(product_id, _, _, _, _)| *product_id);

        let mut item_rows: Vec<(i32, String, i32, f64, Option<String>)> =
            Vec::with_capacity(sorted_items.len());

        // 类似: item_row :=make([]models.OrderItem,0,len(sortedItems))

        for (product_id, product_name, quantity, unit_price, spec_info) in sorted_items.into_iter()
        {
            let result = sqlx::query(
                r#"UPDATE wx_inventory
                   SET available_quantity = available_quantity - ?,
                       frozen_quantity = frozen_quantity + ?,
                       version = version + 1
                   WHERE sku_id = ? AND warehouse_id = 1 AND available_quantity >= ?"#,
            )
            .bind(quantity)
            .bind(quantity)
            .bind(product_id)
            .bind(quantity)
            .execute(&mut *tx)
            .await?;

            if result.rows_affected() == 0 {
                return Err(sqlx::Error::RowNotFound);
            }

            sqlx::query(
                r#"INSERT INTO wx_inventory_log
                   (sku_id, change_type, change_quantity, available_quantity_after, frozen_quantity_after, related_order_id, operator, remark)
                   SELECT sku_id, 2, ?, available_quantity, frozen_quantity, ?, ?, ?
                   FROM wx_inventory
                   WHERE sku_id = ? AND warehouse_id = 1"#,
            )
            .bind(quantity)
            .bind(&order_id)
            .bind("system")
            .bind("order occupy")
            .bind(product_id)
            .execute(&mut *tx)
            .await?;

            item_rows.push((product_id, product_name, quantity, unit_price, spec_info));
        }

        if !item_rows.is_empty() {
            let mut qb: QueryBuilder<MySql> = QueryBuilder::new(
                "INSERT INTO wx_order_items (order_id, product_id, product_name, quantity, unit_price, spec_info) ",
            );
            qb.push_values(
                item_rows.iter(),
                |mut b, (product_id, product_name, quantity, unit_price, spec_info)| {
                    b.push_bind(&order_id)
                        .push_bind(product_id)
                        .push_bind(product_name)
                        .push_bind(quantity)
                        .push_bind(unit_price)
                        .push_bind(spec_info);
                },
            );
            qb.build().execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn delete_order_internal(&self, order_id: &str) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(r#"UPDATE wx_orders SET deleted_at = CURRENT_TIMESTAMP WHERE order_id = ?"#)
            .bind(order_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(r#"UPDATE wx_order_items SET is_deleted = 1 WHERE order_id = ?"#)
            .bind(order_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn update_order_internal(
        &self,
        order_id: &str,
        pay_amount: Option<f64>,
        order_status: Option<i8>,
        consignee_info: Option<Value>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"UPDATE wx_orders
               SET pay_amount = COALESCE(?, pay_amount),
                   order_status = COALESCE(?, order_status),
                   consignee_info = COALESCE(?, consignee_info)
               WHERE order_id = ?"#,
        )
        .bind(pay_amount)
        .bind(order_status)
        .bind(consignee_info)
        .bind(order_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_order_internal(
        &self,
        order_id: &str,
    ) -> Result<Option<models::Order>, sqlx::Error> {
        sqlx::query_as::<_, models::Order>(
            r#"SELECT order_id, user_id, total_amount, pay_amount, order_status, consignee_info, created_at, updated_at, deleted_at
               FROM wx_orders WHERE order_id = ?"#,
        )
            .bind(order_id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn get_order_items_internal(
        &self,
        order_id: &str,
    ) -> Result<Vec<models::OrderItem>, sqlx::Error> {
        sqlx::query_as::<_, models::OrderItem>(
            r#"SELECT item_id, order_id, product_id, product_name, quantity, unit_price, subtotal, spec_info, created_at, is_deleted
               FROM wx_order_items WHERE order_id = ? AND is_deleted = 0"#,
        )
            .bind(order_id)
            .fetch_all(&self.pool)
            .await
    }

    async fn cancel_order_internal(&self, order_id: &str) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // 1. 获取订单项，用于恢复库存
        let items: Vec<(i32, i32)> = sqlx::query_as(
            r#"SELECT product_id, quantity FROM wx_order_items WHERE order_id = ? AND is_deleted = 0"#,
        )
        .bind(order_id)
        .fetch_all(&mut *tx)
        .await?;

        // 2. 更新订单状态为已取消 (order_status = 4, 假设 4 代表已取消)
        // 只有待付款(0)和待发货(1)的订单可以取消
        let result = sqlx::query(
            r#"UPDATE wx_orders SET order_status = 4 WHERE order_id = ? AND order_status IN (0, 1)"#,
        )
        .bind(order_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            // 订单不存在或状态不符合取消条件
            return Err(sqlx::Error::RowNotFound);
        }

        // 3. 恢复库存
        for (product_id, quantity) in items {
            // 恢复可用库存，减少冻结库存（如果是下单减库存模式）
            // 这里假设是下单锁库存，所以取消订单需要释放冻结库存并加回可用库存
            // 注意：具体逻辑取决于库存扣减策略，这里假设是预占库存模式
            sqlx::query(
                r#"UPDATE wx_inventory
                   SET available_quantity = available_quantity + ?,
                       frozen_quantity = frozen_quantity - ?,
                       version = version + 1
                   WHERE sku_id = ? AND warehouse_id = 1"#,
            )
            .bind(quantity)
            .bind(quantity)
            .bind(product_id)
            .execute(&mut *tx)
            .await?;

            // 记录库存变动日志
            sqlx::query(
                r#"INSERT INTO wx_inventory_log
                   (sku_id, change_type, change_quantity, available_quantity_after, frozen_quantity_after, related_order_id, operator, remark)
                   SELECT sku_id, 3, ?, available_quantity, frozen_quantity, ?, ?, ?
                   FROM wx_inventory
                   WHERE sku_id = ? AND warehouse_id = 1"#,
            )
            .bind(quantity)
            .bind(order_id)
            .bind("system")
            .bind("order cancel release")
            .bind(product_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

impl OrderRepo for OrderRepository {
    fn clone_box(&self) -> Box<dyn OrderRepo> {
        Box::new(OrderRepository {
            pool: self.pool.clone(),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn create_order_with_items_blocking<'a>(
        &'a self,
        params: CreateOrderWithItemsParams,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), sqlx::Error>> + Send + 'a>>
    {
        Box::pin(self.create_order_with_items_internal(params))
    }

    fn delete_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), sqlx::Error>> + Send + 'a>>
    {
        Box::pin(self.delete_order_internal(order_id))
    }

    fn update_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
        pay_amount: Option<f64>,
        order_status: Option<i8>,
        consignee_info: Option<Value>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), sqlx::Error>> + Send + 'a>>
    {
        Box::pin(self.update_order_internal(order_id, pay_amount, order_status, consignee_info))
    }

    fn get_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Option<models::Order>, sqlx::Error>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(self.get_order_internal(order_id))
    }

    fn get_order_items_blocking<'a>(
        &'a self,
        order_id: &'a str,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<models::OrderItem>, sqlx::Error>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(self.get_order_items_internal(order_id))
    }

    fn cancel_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), sqlx::Error>> + Send + 'a>>
    {
        Box::pin(self.cancel_order_internal(order_id))
    }
}
