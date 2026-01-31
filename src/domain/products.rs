use crate::models;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use sqlx::types::BigDecimal;

pub struct CreateProductParams {
    pub product_name: String,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    pub spec_template: Option<Value>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub base_price: BigDecimal,
}

pub struct UpdateProductParams {
    pub product_name: Option<String>,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    pub spec_template: Option<Value>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub base_price: Option<BigDecimal>,
    pub status: Option<i8>,
}

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
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<models::Product>, u64), sqlx::Error>> + Send + 'a>>;
}

impl Clone for Box<dyn ProductRepo> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
