use crate::models;
use crate::models::dto::params::{CreateOrderWithItemsParams, CreateProductParams, UpdateInventoryParams, UpdateProductParams};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub type RepoResult<T> = Pin<Box<dyn Future<Output = Result<T, sqlx::Error>> + Send>>;
pub type RepoResultWithLifetime<'a, T> = Pin<Box<dyn Future<Output = Result<T, sqlx::Error>> + Send + 'a>>;

pub trait ProductRepo: Send + Sync {
    fn clone_box(&self) -> Box<dyn ProductRepo>;
    fn as_any(&self) -> &dyn std::any::Any;

    fn create_product_blocking<'a>(
        &'a self,
        params: CreateProductParams,
    ) -> RepoResultWithLifetime<'a, i32>;

    fn update_product_blocking<'a>(
        &'a self,
        product_id: i32,
        params: UpdateProductParams,
    ) -> RepoResultWithLifetime<'a, ()>;

    fn get_product_blocking<'a>(
        &'a self,
        product_id: i32,
    ) -> RepoResultWithLifetime<'a, Option<models::Product>>;

    fn list_products_blocking<'a>(
        &'a self,
        page: u32,
        page_size: u32,
        product_name: Option<String>,
    ) -> RepoResultWithLifetime<'a, (Vec<models::Product>, u64)>;
}

impl Clone for Box<dyn ProductRepo> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait OrderRepo: Send + Sync {
    fn clone_box(&self) -> Box<dyn OrderRepo>;
    fn as_any(&self) -> &dyn std::any::Any;

    fn create_order_with_items_blocking<'a>(
        &'a self,
        params: CreateOrderWithItemsParams,
    ) -> RepoResultWithLifetime<'a, ()>;

    fn delete_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
    ) -> RepoResultWithLifetime<'a, ()>;

    fn update_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
        pay_amount: Option<f64>,
        order_status: Option<i8>,
        consignee_info: Option<Value>,
    ) -> RepoResultWithLifetime<'a, ()>;

    fn get_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
    ) -> RepoResultWithLifetime<'a, Option<models::Order>>;

    fn get_order_items_blocking<'a>(
        &'a self,
        order_id: &'a str,
    ) -> RepoResultWithLifetime<'a, Vec<models::OrderItem>>;

    fn cancel_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
    ) -> RepoResultWithLifetime<'a, ()>;
}

impl Clone for Box<dyn OrderRepo> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait UserRepo: Send + Sync {
    fn clone_box(&self) -> Box<dyn UserRepo>;
    fn as_any(&self) -> &dyn std::any::Any;

    fn find_by_username_blocking<'a>(
        &'a self,
        username: &'a str,
    ) -> RepoResultWithLifetime<'a, Option<models::User>>;
    fn find_user_by_id_blocking<'a>(
        &'a self,
        id: u32,
    ) -> RepoResultWithLifetime<'a, Option<models::User>>;
}

impl Clone for Box<dyn UserRepo> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait InventoryRepo: Send + Sync {
    fn list_inventory_blocking(
        &self,
        page: u32,
        page_size: u32,
        product_name: Option<String>,
    ) -> RepoResult<(Vec<models::Inventory>, u64)>;

    fn update_inventory_blocking(
        &self,
        inv_id: i32,
        params: UpdateInventoryParams,
    ) -> RepoResult<()>;
    
    fn get_inventory_blocking(
        &self,
        inv_id: i32,
    ) -> RepoResult<Option<models::Inventory>>;
}
