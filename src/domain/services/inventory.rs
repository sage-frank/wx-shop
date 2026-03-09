use crate::infra::repository::traits::InventoryRepo;
use crate::domain::models::dto::params::UpdateInventoryParams;
use crate::domain::models::Inventory;
use crate::domain::services::{Future, Pin, ServiceError, ServiceResultWithLifetime};


use std::sync::Arc;
use tracing::instrument;

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
        product_name: Option<String>,
    ) -> ServiceResultWithLifetime<'a, (Vec<Inventory>, u64)>;

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
    #[instrument(level = "debug", skip(self, product_name), fields(page, page_size))]
    fn list_inventory<'a>(
        &'a self,
        page: u32,
        page_size: u32,
        product_name: Option<String>,
    ) -> ServiceResultWithLifetime<'a, (Vec<Inventory>, u64)> {
        Box::pin(async move {
            self.repo
                .list_inventory_blocking(page, page_size, product_name)
                .await
                .map_err(ServiceError::Database)
        })
    }

    #[instrument(level = "debug", skip(self, params), fields(inv_id))]
    fn update_inventory<'a>(
        &'a self,
        inv_id: i32,
        params: UpdateInventoryParams,
    ) -> ServiceResultWithLifetime<'a, ()> {
        Box::pin(async move {
            self.repo
                .update_inventory_blocking(inv_id, params)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => ServiceError::NotFound("Inventory not found or version mismatch".into()),
                    _ => ServiceError::Database(e),
                })
        })
    }

    #[instrument(level = "debug", skip(self, inv_id), fields(inv_id))]
    fn get_inventory<'a>(
        &'a self,
        inv_id: i32,
    ) -> ServiceResultWithLifetime<'a, Inventory> {
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
    repo: Arc<crate::infra::repository::inventory::InventoryRepository>,
) -> Arc<dyn InventoryService> {
    Arc::new(InventoryServiceImpl::new(repo)) as Arc<dyn InventoryService>
}
