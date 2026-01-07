use sqlx::FromRow;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};

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
    pub total_amount: f64,
    pub pay_amount: Option<f64>,
    pub order_status: i8,
    pub consignee_info: Option<serde_json::Value>,
    pub created_at: Option<DateTime<Local>>,
    pub updated_at: Option<DateTime<Local>>,
    pub deleted_at: Option<DateTime<Local>>,
}


#[derive(FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub item_id: i32,
    pub order_id: String,
    pub product_id: i32,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub subtotal: f64,
    pub spec_info: Option<String>,
    pub created_at: Option<DateTime<Local>>,
    pub is_deleted: i8,
}
