use sqlx::{MySql, Pool};
use std::sync::Arc;

use crate::models;
// 实际项目中这里会有数据库连接池，这里我们只使用一个空结构体
pub struct UserRepository {
    pool: Pool<MySql>,
}

impl UserRepository {
    pub fn new(pool: Pool<MySql>) -> Arc<Self> {
        Arc::new(Self { pool })
    }
    // 模拟数据库查询
    pub async fn find_user_by_id(&self, id: u64) -> Option<models::User> {
        // println!("-> Repo: Looking up user {} in DB...", id);
        let result = sqlx::query_as!(models::User, "SELECT name, email FROM users WHERE id = ?", id)
            .fetch_optional(&self.pool)
            .await;

        match result {
            Ok(Some(user)) => Some(user),
            _ => None,
        }

        /*

           Some(models::User{
            name:"xxx".to_string(),
            email: Some("ttt".to_string()),
        })
        */
    }
}
