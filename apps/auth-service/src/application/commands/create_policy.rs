use std::sync::Arc;
use crate::domain::aggregates::{Policy, policy::{Statement, Effect}};
use crate::domain::errors::DomainError;
use crate::domain::repositories::PolicyRepository;

pub struct CreatePolicyCommand {
    pub name: String,
    pub description: Option<String>,
    pub statements: Vec<StatementDto>,
    pub tenant_id: String,
}

pub struct StatementDto {
    pub sid: String,
    pub effect: String,
    pub actions: Vec<String>,
    pub resources: Vec<String>,
}

pub struct CreatePolicyHandler {
    policy_repo: Arc<dyn PolicyRepository>,
}

impl CreatePolicyHandler {
    pub fn new(policy_repo: Arc<dyn PolicyRepository>) -> Self {
        Self { policy_repo }
    }

    pub async fn handle(&self, command: CreatePolicyCommand) -> Result<Policy, DomainError> {
        if self.policy_repo.find_by_name(&command.name, &command.tenant_id).await?.is_some() {
            return Err(DomainError::AlreadyExists(format!("Policy with name {} already exists", command.name)));
        }

        let statements = command.statements.into_iter().map(|s| {
            let effect = match s.effect.to_lowercase().as_str() {
                "allow" => Effect::Allow,
                "deny" => Effect::Deny,
                _ => Effect::Deny, // Generate error optionally, defaulting to Deny for safety
            };
            Statement {
                sid: s.sid,
                effect,
                actions: s.actions,
                resources: s.resources,
            }
        }).collect();

        let policy = Policy::new(
            command.name,
            command.description,
            statements,
            command.tenant_id,
        )?;

        self.policy_repo.save(&policy).await
    }
}
