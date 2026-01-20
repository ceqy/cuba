use crate::domain::{Permission, Role};
use async_trait::async_trait;

#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Role>, anyhow::Error>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Role>, anyhow::Error>;
    async fn save(&self, role: &Role) -> Result<(), anyhow::Error>;
    async fn delete(&self, id: &str) -> anyhow::Result<()>;
    async fn find_by_user_id(&self, user_id: &str) -> anyhow::Result<Vec<Role>>;
    async fn grant_permissions(
        &self,
        role_id: &str,
        permission_ids: &[String],
    ) -> anyhow::Result<()>;
    async fn assign_to_user(&self, user_id: &str, role_id: &str) -> Result<(), anyhow::Error>;
    async fn remove_from_user(&self, user_id: &str, role_id: &str) -> Result<(), anyhow::Error>;
}

#[async_trait]
pub trait PermissionRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<Permission>, anyhow::Error>;
    async fn find_by_user_id(&self, user_id: &str) -> Result<Vec<Permission>, anyhow::Error>;
}
