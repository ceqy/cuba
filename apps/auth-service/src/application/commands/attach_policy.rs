use std::sync::Arc;
use crate::domain::errors::DomainError;
use crate::domain::repositories::PolicyRepository;

pub struct AttachPolicyToRoleCommand {
    pub policy_id: String,
    pub role_id: String,
}

pub struct AttachPolicyToRoleHandler {
    policy_repo: Arc<dyn PolicyRepository>,
}

impl AttachPolicyToRoleHandler {
    pub fn new(policy_repo: Arc<dyn PolicyRepository>) -> Self {
        Self { policy_repo }
    }

    pub async fn handle(&self, command: AttachPolicyToRoleCommand) -> Result<(), DomainError> {
        // 可以添加验证 policy_id 和 role_id 是否存在的逻辑
        // 目前为了性能，直接尝试关联
        self.policy_repo.attach_to_role(&command.policy_id, &command.role_id).await
    }
}

pub struct AttachPolicyToUserCommand {
    pub policy_id: String,
    pub user_id: String,
}

pub struct AttachPolicyToUserHandler {
    policy_repo: Arc<dyn PolicyRepository>,
}

impl AttachPolicyToUserHandler {
    pub fn new(policy_repo: Arc<dyn PolicyRepository>) -> Self {
        Self { policy_repo }
    }

    pub async fn handle(&self, command: AttachPolicyToUserCommand) -> Result<(), DomainError> {
        self.policy_repo.attach_to_user(&command.policy_id, &command.user_id).await
    }
}
