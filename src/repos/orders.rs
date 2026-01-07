use sqlx::{MySql, Pool};
use std::sync::Arc;
use crate::models;
use crate::domain::orders::OrderRepo;
use serde_json::Value;

pub struct OrderRepository {
    pool: Pool<MySql>,
}

impl OrderRepository {
    pub fn new(pool: Pool<MySql>) -> Arc<Self> {
        Arc::new(Self { pool })
    }

    async fn create_order_with_items_internal(
        &self,
        order_id: &str,
        user_id: i32,
        total_amount: f64,
        pay_amount: Option<f64>,
        order_status: i8,
        consignee_info: Option<Value>,
        items: Vec<(i32, String, i32, f64, Option<String>)>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"INSERT INTO orders (order_id, user_id, total_amount, pay_amount, order_status, consignee_info)
               VALUES (?, ?, ?, ?, ?, ?)"#,
        )
            .bind(order_id)
            .bind(user_id)
            .bind(total_amount)
            .bind(pay_amount)
            .bind(order_status)
            .bind(consignee_info)
            .execute(&mut *tx)
            .await?;

        for (product_id, product_name, quantity, unit_price, spec_info) in items.into_iter() {
            sqlx::query(
                r#"INSERT INTO order_items (order_id, product_id, product_name, quantity, unit_price, spec_info)
                   VALUES (?, ?, ?, ?, ?, ?)"#,
            )
                .bind(order_id)
                .bind(product_id)
                .bind(product_name)
                .bind(quantity)
                .bind(unit_price)
                .bind(spec_info)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn delete_order_internal(&self, order_id: &str) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(r#"UPDATE orders SET deleted_at = CURRENT_TIMESTAMP WHERE order_id = ?"#)
            .bind(order_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(r#"UPDATE order_items SET is_deleted = 1 WHERE order_id = ?"#)
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
            r#"UPDATE orders
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

    async fn get_order_internal(&self, order_id: &str) -> Result<Option<models::Order>, sqlx::Error> {
        sqlx::query_as::<_, models::Order>(
            r#"SELECT order_id, user_id, total_amount, pay_amount, order_status, consignee_info, created_at, updated_at, deleted_at
               FROM orders WHERE order_id = ?"#,
        )
            .bind(order_id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn get_order_items_internal(&self, order_id: &str) -> Result<Vec<models::OrderItem>, sqlx::Error> {
        sqlx::query_as::<_, models::OrderItem>(
            r#"SELECT item_id, order_id, product_id, product_name, quantity, unit_price, subtotal, spec_info, created_at, is_deleted
               FROM order_items WHERE order_id = ? AND is_deleted = 0"#,
        )
            .bind(order_id)
            .fetch_all(&self.pool)
            .await
    }
}

impl OrderRepo for OrderRepository {
    fn clone_box(&self) -> Box<dyn OrderRepo> {
        Box::new(OrderRepository { pool: self.pool.clone() })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn create_order_with_items_blocking<'a>(
        &'a self,
        order_id: &'a str,
        user_id: i32,
        total_amount: f64,
        pay_amount: Option<f64>,
        order_status: i8,
        consignee_info: Option<Value>,
        items: Vec<(i32, String, i32, f64, Option<String>)>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), sqlx::Error>> + Send + 'a>> {
        Box::pin(self.create_order_with_items_internal(order_id, user_id, total_amount, pay_amount, order_status, consignee_info, items))
    }

    fn delete_order_blocking<'a>(&'a self, order_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), sqlx::Error>> + Send + 'a>> {
        Box::pin(self.delete_order_internal(order_id))
    }

    fn update_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
        pay_amount: Option<f64>,
        order_status: Option<i8>,
        consignee_info: Option<Value>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), sqlx::Error>> + Send + 'a>> {
        Box::pin(self.update_order_internal(order_id, pay_amount, order_status, consignee_info))
    }

    fn get_order_blocking<'a>(&'a self, order_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<models::Order>, sqlx::Error>> + Send + 'a>> {
        Box::pin(self.get_order_internal(order_id))
    }

    fn get_order_items_blocking<'a>(&'a self, order_id: &'a str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<models::OrderItem>, sqlx::Error>> + Send + 'a>> {
        Box::pin(self.get_order_items_internal(order_id))
    }
}
