use crate::models;
use crate::models::dto::params::{CreateOrderWithItemsParams, CreateProductParams, UpdateInventoryParams, UpdateProductParams};
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub trait ProductRepo: Send + Sync {
    fn clone_box(&self) -> Box<dyn ProductRepo>;
    fn as_any(&self) -> &dyn std::any::Any;

    fn create_product_blocking<'a>(
        &'a self,
        params: CreateProductParams,
    ) -> Pin<Box<dyn Future<Output = Result<i32, sqlx::Error>> + Send + 'a>>;

    fn update_product_blocking<'a>(
        &'a self,
        product_id: i32,
        params: UpdateProductParams,
    ) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'a>>;

    fn get_product_blocking<'a>(
        &'a self,
        product_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Option<models::Product>, sqlx::Error>> + Send + 'a>>;

    fn list_products_blocking<'a>(
        &'a self,
        page: u32,
        page_size: u32,
        product_name: Option<String>,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<models::Product>, u64), sqlx::Error>> + Send + 'a>>;
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
    ) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'a>>;

    fn delete_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'a>>;

    fn update_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
        pay_amount: Option<f64>,
        order_status: Option<i8>,
        consignee_info: Option<Value>,
    ) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'a>>;

    fn get_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Option<models::Order>, sqlx::Error>> + Send + 'a>>;

    fn get_order_items_blocking<'a>(
        &'a self,
        order_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<models::OrderItem>, sqlx::Error>> + Send + 'a>>;

    fn cancel_order_blocking<'a>(
        &'a self,
        order_id: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send + 'a>>;
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
    ) -> Pin<Box<dyn Future<Output = Result<Option<models::User>, sqlx::Error>> + Send + 'a>>;
    fn find_user_by_id_blocking<'a>(
        &'a self,
        id: u32,
    ) -> Pin<Box<dyn Future<Output = Result<Option<models::User>, sqlx::Error>> + Send + 'a>>;
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
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<models::Inventory>, u64), sqlx::Error>> + Send>>;

    fn update_inventory_blocking(
        &self,
        inv_id: i32,
        params: UpdateInventoryParams,
    ) -> Pin<Box<dyn Future<Output = Result<(), sqlx::Error>> + Send>>;
    
    fn get_inventory_blocking(
        &self,
        inv_id: i32,
    ) -> Pin<Box<dyn Future<Output = Result<Option<models::Inventory>, sqlx::Error>> + Send>>;
}
