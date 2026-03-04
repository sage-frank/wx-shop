use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

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
