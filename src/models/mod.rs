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
    pub created_at: Option<DateTime<Local>>,
    pub updated_at: Option<DateTime<Local>>,
}