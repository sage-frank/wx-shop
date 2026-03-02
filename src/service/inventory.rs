use crate::domain::inventory::{InventoryRepo, UpdateInventoryParams};
use crate::models::Inventory;
use crate::service::ServiceError;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct InventoryServiceImpl<R: InventoryRepo + 'static> {
    repo: Arc<R>,
}

impl<R: InventoryRepo + 'static> InventoryServiceImpl<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

pub trait InventoryService: Send + Sync {
    fn list_inventory<'a>(
        &'a self,
        page: u32,
        page_size: u32,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<Inventory>, u64), ServiceError>> + Send + 'a>>;

    fn update_inventory<'a>(
        &'a self,
        inv_id: i32,
        params: UpdateInventoryParams,
    ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>>;

    fn get_inventory<'a>(
        &'a self,
        inv_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Inventory, ServiceError>> + Send + 'a>>;
}

impl<R: InventoryRepo> InventoryService for InventoryServiceImpl<R> {
    fn list_inventory<'a>(
        &'a self,
        page: u32,
        page_size: u32,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<Inventory>, u64), ServiceError>> + Send + 'a>> {
        Box::pin(async move {
            self.repo
                .list_inventory_blocking(page, page_size)
                .await
                .map_err(ServiceError::Database)
        })
    }

    fn update_inventory<'a>(
        &'a self,
        inv_id: i32,
        params: UpdateInventoryParams,
    ) -> Pin<Box<dyn Future<Output = Result<(), ServiceError>> + Send + 'a>> {
        Box::pin(async move {
            // Check existence first if we want to distinguish not found vs conflict, 
            // but repo handles optimistic lock by returning RowNotFound (mapped to Database error)
            // or we can handle it here.
            // For now simple pass through.
            self.repo
                .update_inventory_blocking(inv_id, params)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => ServiceError::NotFound("Inventory not found or version mismatch".into()),
                    _ => ServiceError::Database(e),
                })
        })
    }

    fn get_inventory<'a>(
        &'a self,
        inv_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Inventory, ServiceError>> + Send + 'a>> {
        Box::pin(async move {
            let opt = self
                .repo
                .get_inventory_blocking(inv_id)
                .await
                .map_err(ServiceError::Database)?;
            opt.ok_or_else(|| ServiceError::NotFound("Inventory not found".into()))
        })
    }
}

pub fn new_inventory_service(
    repo: Arc<crate::repository::inventory::InventoryRepository>,
) -> Arc<dyn InventoryService> {
    Arc::new(InventoryServiceImpl::new(repo)) as Arc<dyn InventoryService>
}
