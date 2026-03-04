use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::BigDecimal;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateProductParams {
    pub product_name: String,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    pub spec_template: Option<Value>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub base_price: BigDecimal,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreateOrderWithItemsParams {
    pub order_id: String,
    pub user_id: i32,
    pub total_amount: f64,
    pub pay_amount: Option<f64>,
    pub order_status: i8,
    pub consignee_info: Option<Value>,
    pub items: Vec<(i32, String, i32, f64, Option<String>)>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdateInventoryParams {
    pub available_quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
    pub version: i32,
}
