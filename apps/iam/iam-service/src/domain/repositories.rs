use async_trait::async_trait;
use crate::domain::aggregates::user::{User, UserId};
use cuba_core::repository::Repository;
use anyhow::Result;

#[async_trait]
pub trait UserRepository: Repository<User, Id = UserId> {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
}
