use async_trait::async_trait;
use crate::domain::aggregates::user::{User, UserId};
use crate::domain::entities::{Role, Permission};
use cuba_core::repository::Repository;
use anyhow::Result;

#[async_trait]
pub trait UserRepository: Repository<User, Id = UserId> {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
}

#[async_trait]
pub trait RoleRepository: Repository<Role, Id = String> {
    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Vec<Role>>;
}

#[async_trait]
pub trait PermissionRepository: Repository<Permission, Id = String> {
    async fn find_by_role_id(&self, role_id: &str) -> Result<Vec<Permission>>;
    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Vec<Permission>>;
}
