use async_trait::async_trait;
use crate::domain::aggregates::Policy;
use crate::domain::errors::DomainError;

#[async_trait]
pub trait PolicyRepository: Send + Sync {
    async fn save(&self, policy: &Policy) -> Result<Policy, DomainError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<Policy>, DomainError>;
    async fn find_by_name(&self, name: &str, tenant_id: &str) -> Result<Option<Policy>, DomainError>;
    async fn delete(&self, id: &str) -> Result<(), DomainError>;
    
    // 附件相关
    async fn attach_to_role(&self, policy_id: &str, role_id: &str) -> Result<(), DomainError>;
    async fn attach_to_user(&self, policy_id: &str, user_id: &str) -> Result<(), DomainError>;
}
