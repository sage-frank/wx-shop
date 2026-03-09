use std::sync::Arc;
use axum::extract::FromRef;
use crate::domain::services::inventory::InventoryService;
use crate::domain::services::orders::OrderService;
use crate::domain::services::products::ProductService;
use crate::domain::services::users::UserService;

#[derive(Clone)]
pub struct AppState {
    pub user_service: Arc<dyn UserService>,
    pub order_service: Arc<dyn OrderService>,
    pub product_service: Arc<dyn ProductService>,
    pub inventory_service: Arc<dyn InventoryService>,
}

impl FromRef<AppState> for Arc<dyn UserService> {
    fn from_ref(state: &AppState) -> Self {
        state.user_service.clone()
    }
}
