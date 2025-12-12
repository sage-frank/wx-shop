use crate::repos::users::UserRepository;
use std::sync::Arc;
use crate::models;
use sha2::{Sha256, Digest};
use crate::service::ServiceError;

#[derive(Clone)]
pub struct UserService {
    repo: Arc<UserRepository>,
}

impl UserService {
    pub fn new(repo: Arc<UserRepository>) -> Self { // Removed Arc wrapper for Self, usually service is cloned or Arc-ed in main.
        Self { repo }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<models::User, String> {
        let user = self.repo.find_by_username(username).await
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
    }

    pub async fn find_user_by_id(&self, id:u32) -> Result<models::User, ServiceError> {
        let user_opt = self.repo.find_user_by_id(id).await?;
        let user = user_opt.ok_or_else(|| ServiceError::NotFound(format!("User with ID {} not found", id)))?;
        Ok(user)
    }
    
}
