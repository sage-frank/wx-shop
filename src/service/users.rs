use crate::repos::users::UserRepository;
use std::sync::Arc;
use crate::models;
pub struct UserService {
    // Service 依赖 Repos，通过 Arc 引用
    repo: Arc<UserRepository>,
}

impl UserService {
    pub fn new(repo: Arc<UserRepository>) -> Arc<Self> {
        Arc::new(Self { repo })
    }

    // 业务逻辑：根据 ID 获取用户
    pub async fn get_user(&self, id: u64) -> Option<models::User> {
        // 调用 Repository
        self.repo.find_user_by_id(id).await
    }
}
