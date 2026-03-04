use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::BigDecimal;

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
