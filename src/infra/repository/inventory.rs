use crate::infra::repository::traits::InventoryRepo;
use crate::domain::models::dto::params::UpdateInventoryParams;
use crate::domain::models::Inventory;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;

pub struct InventoryRepository {
    pool: Pool<MySql>,
}

impl InventoryRepository {
    pub fn new(pool: Pool<MySql>) -> Arc<Self> {
        Arc::new(Self { pool })
    }
}

impl InventoryRepo for InventoryRepository {
    fn list_inventory_blocking(
        &self,
        page: u32,
        page_size: u32,
        product_name: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<Inventory>, u64), sqlx::Error>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let page = if page == 0 { 1 } else { page };
            let page_size = if page_size > 100 { 100 } else { page_size };
            let offset = (page - 1) * page_size;
            
            let mut count_query = sqlx::QueryBuilder::<MySql>::new("SELECT COUNT(*) FROM wx_inventory i LEFT JOIN wx_products p ON i.sku_id = p.product_id WHERE 1=1");
            if let Some(ref name) = product_name {
                count_query.push(" AND p.product_name LIKE ");
                count_query.push_bind(format!("%{}%", name));
            }
            let total: (i64,) = count_query.build_query_as().fetch_one(&pool).await?;
                
            let mut query = sqlx::QueryBuilder::<MySql>::new(r#"
                SELECT i.*, p.product_name 
                FROM wx_inventory i 
                LEFT JOIN wx_products p ON i.sku_id = p.product_id 
                WHERE 1=1
            "#);
            
            if let Some(name) = product_name {
                query.push(" AND p.product_name LIKE ");
                query.push_bind(format!("%{}%", name));
            }
            
            query.push(" LIMIT ");
            query.push_bind(page_size);
            query.push(" OFFSET ");
            query.push_bind(offset);

            let inventory: Vec<Inventory> = query.build_query_as().fetch_all(&pool).await?;
                
            Ok((inventory, total.0 as u64))
        })
    }

    fn update_inventory_blocking(
        &self,
        inv_id: i32,
        params: UpdateInventoryParams,
    ) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let mut qb = sqlx::QueryBuilder::new("UPDATE wx_inventory SET version = version + 1");
            
            if let Some(qty) = params.available_quantity {
                qb.push(", available_quantity = ");
                qb.push_bind(qty);
            }
            if let Some(threshold) = params.low_stock_threshold {
                qb.push(", low_stock_threshold = ");
                qb.push_bind(threshold);
            }
            
            qb.push(" WHERE inv_id = ");
            qb.push_bind(inv_id);
            qb.push(" AND version = ");
            qb.push_bind(params.version);
            
            let result = qb.build().execute(&pool).await?;
            
            if result.rows_affected() == 0 {
                // Check if it exists but version mismatch
                let exists: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM wx_inventory WHERE inv_id = ?")
                    .bind(inv_id)
                    .fetch_optional(&pool)
                    .await?;
                    
                if exists.is_none() {
                    return Err(sqlx::Error::RowNotFound);
                } else {
                    // Version mismatch, treat as protocol error or custom error? 
                    // SQLx doesn't have optimistic lock error, so we can return RowNotFound or handle in Service
                    // Returning RowNotFound is confusing if row exists. 
                    // But standard sqlx update returning 0 rows means nothing updated.
                    return Err(sqlx::Error::RowNotFound); // Simplified for now
                }
            }
            
            Ok(())
        })
    }

    fn get_inventory_blocking(
        &self,
        inv_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Inventory>, sqlx::Error>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let query = r#"
                SELECT i.*, p.product_name 
                FROM wx_inventory i 
                LEFT JOIN wx_products p ON i.sku_id = p.product_id 
                WHERE i.inv_id = ?
            "#;
            let inventory = sqlx::query_as(query)
                .bind(inv_id)
                .fetch_optional(&pool)
                .await?;
            Ok(inventory)
        })
    }
}
