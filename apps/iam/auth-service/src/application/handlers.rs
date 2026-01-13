use crate::domain::User;
use crate::domain::repositories::{UserRepository, UserSessionRepository};
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
    session_repository: Arc<dyn UserSessionRepository<Id = String> + Send + Sync>,
    rbac_client: Arc<crate::infrastructure::rbac_client::RbacClient>,
    password_service: Arc<BcryptPasswordService>,
    token_service: Arc<JwtTokenService>,
}

impl LoginUserHandler {
    pub fn new(
        user_repository: Arc<dyn UserRepository<Id = String> + Send + Sync>,
        session_repository: Arc<dyn UserSessionRepository<Id = String> + Send + Sync>,
        rbac_client: Arc<crate::infrastructure::rbac_client::RbacClient>,
        password_service: Arc<BcryptPasswordService>,
        token_service: Arc<JwtTokenService>,
    ) -> Self {
        Self {
            user_repository,
            session_repository,
            rbac_client,
            password_service,
            token_service,
        }
    }

    pub async fn handle(
        &self, 
        username: String, 
        password: String, 
        tenant_id: String,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Result<(String, String, String, User), anyhow::Error> {
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

        // Fetch Roles from RBAC Service
        let roles = self.rbac_client.get_user_roles(&user.id).await.unwrap_or_default();

        // Generate Token
        let token = self.token_service.generate_token(&user.id, &user.tenant_id, roles)?;
        
        // Generate Refresh Token
        let refresh_token = self.token_service.generate_refresh_token();
        
        // Create Session
        let expires_at = chrono::Utc::now() + chrono::Duration::days(30);
        let session = crate::domain::UserSession::new(
            user.id.clone(),
            user.tenant_id.clone(),
            refresh_token.clone(),
            user_agent,
            ip_address,
            expires_at,
        );
        let session_id = session.id.clone();
        
        self.session_repository.save(&session).await?;

        Ok((token, refresh_token, session_id, user))
    }
}

pub struct RefreshTokenHandler {
    user_repository: Arc<dyn UserRepository<Id = String> + Send + Sync>,
    session_repository: Arc<dyn UserSessionRepository<Id = String> + Send + Sync>,
    rbac_client: Arc<crate::infrastructure::rbac_client::RbacClient>,
    token_service: Arc<JwtTokenService>,
}

impl RefreshTokenHandler {
    pub fn new(
        user_repository: Arc<dyn UserRepository<Id = String> + Send + Sync>,
        session_repository: Arc<dyn UserSessionRepository<Id = String> + Send + Sync>,
        rbac_client: Arc<crate::infrastructure::rbac_client::RbacClient>,
        token_service: Arc<JwtTokenService>,
    ) -> Self {
        Self {
            user_repository,
            session_repository,
            rbac_client,
            token_service,
        }
    }

    pub async fn handle(&self, refresh_token: String) -> Result<(String, String), anyhow::Error> {
        // Find session
        let mut session = self.session_repository.find_by_refresh_token(&refresh_token).await?
            .ok_or_else(|| anyhow::anyhow!("Invalid refresh token"))?;

        // Check expiration
        if session.expires_at < chrono::Utc::now() {
            return Err(anyhow::anyhow!("Refresh token expired"));
        }

        // Find user
        let user = self.user_repository.find_by_id(&session.user_id).await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))?;

        // Rotate Refresh Token
        let new_refresh_token = self.token_service.generate_refresh_token();
        session.refresh_token = new_refresh_token.clone();
        session.last_seen_at = chrono::Utc::now();
        
        self.session_repository.save(&session).await?;

        // Fetch Roles
        let roles = self.rbac_client.get_user_roles(&user.id).await.unwrap_or_default();

        // Generate New Access Token
        let access_token = self.token_service.generate_token(&user.id, &user.tenant_id, roles)?;

        Ok((access_token, new_refresh_token))
    }
}
