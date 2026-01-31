use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use crate::models::Inventory;

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateInventoryParams {
    pub available_quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
    pub version: i32,
}

pub trait InventoryRepo: Send + Sync {
    fn list_inventory_blocking(
        &self,
        page: u32,
        page_size: u32,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<Inventory>, u64), sqlx::Error>> + Send>>;

    fn update_inventory_blocking(
        &self,
        inv_id: i32,
        params: UpdateInventoryParams,
    ) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send>>;
    
    fn get_inventory_blocking(
        &self,
        inv_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Option<Inventory>, sqlx::Error>> + Send>>;
}
