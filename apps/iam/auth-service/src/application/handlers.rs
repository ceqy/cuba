use crate::domain::User;
use crate::domain::repositories::UserRepository;
use crate::infrastructure::{BcryptPasswordService, JwtTokenService};
use std::sync::Arc;

pub struct RegisterUserHandler {
    user_repository: Arc<dyn UserRepository<Id = String> + Send + Sync>,
    password_service: Arc<BcryptPasswordService>,
}

impl RegisterUserHandler {
    pub fn new(
        user_repository: Arc<dyn UserRepository<Id = String> + Send + Sync>,
        password_service: Arc<BcryptPasswordService>,
    ) -> Self {
        Self {
            user_repository,
            password_service,
        }
    }

    pub async fn handle(&self, username: String, email: String, password: String, tenant_id: String) -> Result<User, anyhow::Error> {
        // Check if user exists
        if (self.user_repository.find_by_username(&username).await?).is_some() {
            return Err(anyhow::anyhow!("Username already exists"));
        }
        if (self.user_repository.find_by_email(&email).await?).is_some() {
            return Err(anyhow::anyhow!("Email already exists"));
        }

        // Hash password
        let password_hash = self.password_service.hash_password(&password)?;

        // Create User
        let user = User::new(username, email, password_hash, tenant_id);

        // Save
        self.user_repository.save(&user).await?;

        Ok(user)
    }
}

pub struct LoginUserHandler {
    user_repository: Arc<dyn UserRepository<Id = String> + Send + Sync>,
    password_service: Arc<BcryptPasswordService>,
    token_service: Arc<JwtTokenService>,
}

impl LoginUserHandler {
    pub fn new(
        user_repository: Arc<dyn UserRepository<Id = String> + Send + Sync>,
        password_service: Arc<BcryptPasswordService>,
        token_service: Arc<JwtTokenService>,
    ) -> Self {
        Self {
            user_repository,
            password_service,
            token_service,
        }
    }

    pub async fn handle(&self, username: String, password: String, tenant_id: String) -> Result<(String, User), anyhow::Error> {
        // Find user
        let user = self.user_repository.find_by_username(&username).await?
            .ok_or_else(|| anyhow::anyhow!("Invalid credentials"))?;

        // Verify password
        if !self.password_service.verify_password(&password, &user.password_hash)? {
            return Err(anyhow::anyhow!("Invalid credentials"));
        }
        
        // Verify tenant (optional, strictly speaking)
        if user.tenant_id != tenant_id {
             return Err(anyhow::anyhow!("Invalid tenant"));
        }

        // Generate Token
        let token = self.token_service.generate_token(&user.id, &user.tenant_id, user.roles.clone())?;

        Ok((token, user))
    }
}
