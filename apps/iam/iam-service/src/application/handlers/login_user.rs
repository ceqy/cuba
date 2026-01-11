use anyhow::{Result, bail, anyhow};
use async_trait::async_trait;
use cuba_cqrs::QueryHandler;
use std::sync::Arc;
use crate::application::queries::LoginUserQuery;
use crate::domain::repositories::UserRepository;
use crate::domain::services::{PasswordService, TokenService, TokenPair};

pub struct LoginUserDTO {
    pub user_id: String,
    pub username: String,
    pub email: String,
    pub tenant_id: Option<String>,
    pub token_pair: TokenPair,
}

pub struct LoginUserHandler {
    user_repo: Arc<dyn UserRepository>,
    password_service: Arc<dyn PasswordService>,
    token_service: Arc<dyn TokenService>,
}

impl LoginUserHandler {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        password_service: Arc<dyn PasswordService>,
        token_service: Arc<dyn TokenService>,
    ) -> Self {
        Self { user_repo, password_service, token_service }
    }
}

#[async_trait]
impl QueryHandler<LoginUserQuery> for LoginUserHandler {
    type Output = LoginUserDTO;

    async fn handle(&self, query: LoginUserQuery) -> Result<Self::Output> {
        // 1. Find user
        let user = self.user_repo.find_by_username(&query.username).await?
            .ok_or_else(|| anyhow!("Invalid username or password"))?;

        // 2. Verify password
        let matches = self.password_service.verify(&query.password, &user.password_hash).await?;
        if !matches {
            bail!("Invalid username or password");
        }
        
        // 3. Check tenant (if needed)
        
        // 4. Generate tokens
        let token_pair = self.token_service.generate_tokens(
            &user.id.clone().into_inner().to_string(), 
            user.tenant_id.clone()
        )?;

        Ok(LoginUserDTO {
            user_id: user.id.into_inner().to_string(),
            username: user.username,
            email: user.email,
            tenant_id: user.tenant_id,
            token_pair,
        })
    }
}
