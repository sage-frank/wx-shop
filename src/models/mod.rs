use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::BigDecimal;

pub mod dto;

#[derive(FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub username: String,

    #[serde(skip)]
    pub passwd: String,
    #[serde(skip)]
    pub salt: String,
    #[serde(skip)]
    pub created_at: Option<DateTime<Local>>,
    #[serde(skip)]
    pub updated_at: Option<DateTime<Local>>,
}

#[derive(FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub user_id: i32,
    pub total_amount: BigDecimal,
    pub pay_amount: Option<BigDecimal>,
    pub order_status: i8,
    pub consignee_info: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Local>>,
    #[serde(skip)]
    pub updated_at: Option<DateTime<Local>>,
    #[serde(skip)]
    pub deleted_at: Option<DateTime<Local>>,
}

#[derive(FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub item_id: i32,
    pub order_id: String,
    pub product_id: i32,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: BigDecimal,
    pub subtotal: BigDecimal,
    pub spec_info: Option<String>,
    #[serde(skip)]
    pub created_at: Option<DateTime<Local>>,
    #[serde(skip)]
    pub is_deleted: i8,
}

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
    pub warehouse_id: Option<i32>,
    pub available_quantity: i32,
    pub frozen_quantity: i32,
    pub version: i32,
    pub low_stock_threshold: Option<i32>,
    pub updated_at: Option<DateTime<Local>>,
}
