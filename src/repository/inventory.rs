use crate::repository::traits::InventoryRepo;
use crate::models::dto::params::UpdateInventoryParams;
use crate::models::Inventory;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

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
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<Inventory>, u64), sqlx::Error>> + Send>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            let offset = (page - 1) * page_size;
            
            let count_query = "SELECT COUNT(*) FROM wx_inventory";
            let total: (i64,) = sqlx::query_as(count_query)
                .fetch_one(&pool)
                .await?;
                
            let query = "SELECT * FROM wx_inventory LIMIT ? OFFSET ?";
            let inventory: Vec<Inventory> = sqlx::query_as(query)
                .bind(page_size)
                .bind(offset)
                .fetch_all(&pool)
                .await?;
                
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
            let inventory = sqlx::query_as("SELECT * FROM wx_inventory WHERE inv_id = ?")
                .bind(inv_id)
                .fetch_optional(&pool)
                .await?;
            Ok(inventory)
        })
    }
}
