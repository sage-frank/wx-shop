use sqlx::{MySql, Pool};
use std::sync::Arc;
use crate::models;
use std::future::Future;
use std::pin::Pin;
use crate::domain::users::UserRepo;

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

impl UserRepo for UserRepository {
    fn clone_box(&self) -> Box<dyn UserRepo> {
        // UserRepository is cheap to clone because Pool is Clone internally via Arc-like handle
        Box::new(UserRepository { pool: self.pool.clone() })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn find_by_username_blocking<'a>(&'a self, username: &'a str) -> Pin<Box<dyn Future<Output = Result<Option<models::User>, sqlx::Error>> + Send + 'a>> {
        Box::pin(self.find_by_username(username))
    }

    fn find_user_by_id_blocking<'a>(&'a self, id: u32) -> Pin<Box<dyn Future<Output = Result<Option<models::User>, sqlx::Error>> + Send + 'a>> {
        Box::pin(self.find_user_by_id(id))
    }
}
