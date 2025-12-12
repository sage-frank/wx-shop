use sqlx::{MySql, Pool};
use std::sync::Arc;
use crate::models;

pub struct UserRepository {
    pool: Pool<MySql>,
}

impl UserRepository {
    pub fn new(pool: Pool<MySql>) -> Arc<Self> {
        Arc::new(Self { pool })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<models::User>, sqlx::Error> {
        sqlx::query_as::<_, models::User>("SELECT * FROM t_user WHERE username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn find_user_by_id(&self, id:u32) -> Result<Option<models::User>, sqlx::Error> {
        sqlx::query_as::<_, models::User>("SELECT * FROM t_user WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }
}
