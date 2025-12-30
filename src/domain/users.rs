use crate::models;
use std::future::Future;
use std::pin::Pin;

pub trait UserRepo: Send + Sync {
    fn clone_box(&self) -> Box<dyn UserRepo>;
    fn as_any(&self) -> &dyn std::any::Any;

    fn find_by_username_blocking<'a>(&'a self, username: &'a str) -> Pin<Box<dyn Future<Output = Result<Option<models::User>, sqlx::Error>> + Send + 'a>>;
    fn find_user_by_id_blocking<'a>(&'a self, id: u32) -> Pin<Box<dyn Future<Output = Result<Option<models::User>, sqlx::Error>> + Send + 'a>>;
}

impl Clone for Box<dyn UserRepo> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
