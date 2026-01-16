use crate::domain::{User, UserSession};
use cuba_core::repository::Repository;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Repository<User> {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, anyhow::Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, anyhow::Error>;
    async fn list_users(&self, offset: i64, limit: i64) -> Result<(Vec<User>, i64), anyhow::Error>;
    async fn delete(&self, user_id: &str) -> Result<(), anyhow::Error>;
    async fn update(&self, user: &User) -> Result<(), anyhow::Error>;
}

#[async_trait]
pub trait UserSessionRepository: Repository<UserSession> {
    async fn find_by_refresh_token(&self, token: &str) -> Result<Option<UserSession>, anyhow::Error>;
    async fn revoke_by_user_id(&self, user_id: &str) -> Result<(), anyhow::Error>;
}
