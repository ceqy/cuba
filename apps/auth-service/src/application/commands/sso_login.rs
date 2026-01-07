use std::sync::Arc;
use crate::domain::aggregates::User;
use crate::domain::errors::DomainError;
use crate::domain::repositories::UserRepository;
use crate::domain::services::TokenService;
use crate::infrastructure::services::sso_service::SSOService;
use chrono::Utc;

pub struct SSOLoginCommand {
    pub provider: String,
    pub assertion: String,
}

pub struct SSOLoginHandler {
    user_repo: Arc<dyn UserRepository>,
    token_service: Arc<dyn TokenService>,
    sso_service: Arc<SSOService>,
}

impl SSOLoginHandler {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        token_service: Arc<dyn TokenService>,
        sso_service: Arc<SSOService>,
    ) -> Self {
        Self {
            user_repo,
            token_service,
            sso_service,
        }
    }

    pub async fn handle(
        &self,
        command: SSOLoginCommand,
    ) -> Result<(String, String, i64), DomainError> {
        // 1. 获取 SSO 提供商
        let provider = self.sso_service.get_provider(&command.provider)
            .ok_or_else(|| DomainError::InvalidInput(format!("Unsupported SSO provider: {}", command.provider)))?;

        // 2. 验证 Assertion
        let sso_profile = provider.verify_assertion(&command.assertion).await?;

        // 3. 查找或注册用户
        let user = match self.user_repo.find_by_email(&sso_profile.email).await {
            Ok(Some(mut user)) => {
                // 更新登录时间
                user.record_login();
                self.user_repo.save(&mut user).await
                    .map_err(|e| DomainError::InternalError(e.to_string()))?;
                user
            }
            Ok(None) => {
                // 自动注册
                let random_password = uuid::Uuid::new_v4().to_string();
                let mut new_user = User::register(
                    sso_profile.email.clone(), // 使用 email 作为 username
                    sso_profile.email.clone(),
                    &random_password,
                ).map_err(|e| DomainError::InvalidInput(e.to_string()))?;
                
                if let Some(name) = sso_profile.display_name {
                     new_user.update_profile(Some(name), None);
                }
                
                // 这里可能需要默认角色，目前假设无特殊角色
                
                self.user_repo.save(&mut new_user).await
                    .map_err(|e| DomainError::InternalError(e.to_string()))?;
                new_user
            }
            Err(e) => return Err(DomainError::InternalError(e.to_string())),
        };

        if !user.is_active() {
            return Err(DomainError::AuthenticationFailed("Account is disabled".to_string()));
        }

        // 4. 生成 Token
        let roles: Vec<String> = vec![]; // 这里应该从 RoleRepository 获取角色名称，暂时为空
        let permissions: Vec<String> = vec![]; 

        let (access_token, refresh_token, expires_in) = self.token_service.generate_tokens(
            user.id().to_string(),
            String::new(), // Default Tenant
            roles,
            permissions
        ).await
            .map_err(|_| DomainError::InternalError("Failed to generate tokens".to_string()))?;

        Ok((access_token, refresh_token, expires_in))
    }
}
