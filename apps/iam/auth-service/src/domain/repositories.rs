use crate::domain::User;
use cuba_core::repository::Repository;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Repository<User> {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, anyhow::Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, anyhow::Error>;
}
