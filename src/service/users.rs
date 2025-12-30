use crate::repos::users::UserRepository;
use crate::domain::users::UserRepo;
use std::sync::Arc;
use crate::models;
use sha2::{Sha256, Digest};
use crate::service::ServiceError;
use std::future::Future;
use std::pin::Pin;

pub trait UserService: Send + Sync {
    fn login<'a>(&'a self, username: &'a str, password: &'a str) -> Pin<Box<dyn Future<Output = Result<models::User, String>> + Send + 'a>>;
    fn find_user_by_id<'a>(&'a self, id:u32) -> Pin<Box<dyn Future<Output = Result<models::User, ServiceError>> + Send + 'a>>;
}

pub struct UserServiceImpl<R: UserRepo + 'static> {
    repo: Arc<R>,
}

impl<R: UserRepo + 'static> UserServiceImpl<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }
}

impl<R: UserRepo + 'static> UserService for UserServiceImpl<R> {
    fn login<'a>(&'a self, username: &'a str, password: &'a str) -> Pin<Box<dyn Future<Output = Result<models::User, String>> + Send + 'a>> {
        Box::pin(async move {
            let user = self.repo.find_by_username_blocking(username).await
                .map_err(|e| e.to_string())?
                .ok_or("User not found".to_string())?;

            let input_hash = {
                let mut hasher = Sha256::new();
                hasher.update(password.as_bytes());
                hasher.update(user.salt.as_bytes());
                hex::encode(hasher.finalize())
            };

            if input_hash == user.passwd {
                Ok(user)
            } else {
                Err("Invalid password".to_string())
            }
        })
    }

    fn find_user_by_id<'a>(&'a self, id:u32) -> Pin<Box<dyn Future<Output = Result<models::User, ServiceError>> + Send + 'a>> {
        Box::pin(async move {
            let user_opt = self.repo.find_user_by_id_blocking(id).await?;
            let user = user_opt.ok_or_else(|| ServiceError::NotFound(format!("User with ID {} not found", id)))?;
            Ok(user)
        })
    }
}

pub fn new_user_service(repo: Arc<UserRepository>) -> Arc<dyn UserService> {
    Arc::new(UserServiceImpl::new(repo)) as Arc<dyn UserService>
}
