use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::BigDecimal;

#[derive(FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub product_id: i32,
    pub product_name: String,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    pub spec_template: Option<serde_json::Value>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub status: i8,
    pub base_price: BigDecimal,
    pub created_at: Option<DateTime<Local>>,
    pub updated_at: Option<DateTime<Local>>,
}

#[derive(FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub inv_id: i32,
    pub sku_id: i32,
    pub product_name: Option<String>,
    pub warehouse_id: Option<i32>,
    pub available_quantity: i32,
    pub frozen_quantity: i32,
    pub version: i32,
    pub low_stock_threshold: Option<i32>,
    pub updated_at: Option<DateTime<Local>>,
}
