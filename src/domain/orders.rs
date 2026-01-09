use crate::models;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub struct CreateOrderWithItemsParams {
    pub order_id: String,
    pub user_id: i32,
    pub total_amount: f64,
    pub pay_amount: Option<f64>,
    pub order_status: i8,
    pub consignee_info: Option<Value>,
    pub items: Vec<(i32, String, i32, f64, Option<String>)>,
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
}

impl Clone for Box<dyn OrderRepo> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
