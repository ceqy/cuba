//! Verify 2FA Code Handler (Login flow)

use crate::domain::repositories::{UserRepository, RepositoryError};
use crate::domain::value_objects::UserId;
use crate::domain::services::{TokenService, TokenError};
use std::sync::Arc;
use totp_rs::{Algorithm, TOTP};
use base32::Alphabet;

pub struct Verify2FACodeCommand {
    pub temp_token: String,
    pub code: String,
}

pub struct Verify2FACodeResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

pub struct Verify2FACodeHandler {
    user_repo: Arc<dyn UserRepository>,
    token_service: Arc<dyn TokenService>,
}

impl Verify2FACodeHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>, token_service: Arc<dyn TokenService>) -> Self {
        Self { user_repo, token_service }
    }

    pub async fn handle(&self, command: Verify2FACodeCommand) -> Result<Verify2FACodeResponse, String> {
        // 1. 验证临时 Token
        let claims = self.token_service.validate_token(&command.temp_token)
            .map_err(|e| format!("Invalid temporary token: {}", e))?;
        
        if claims.token_type != "temp_2fa" {
            return Err("Invalid token type".to_string());
        }

        let user_id = UserId::parse(&claims.sub).map_err(|e| e.to_string())?;
        let user = self.user_repo.find_by_id(&user_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "User not found".to_string())?;

        // 2. 验证 2FA 代码
        let secret_str = user.tfa_secret().ok_or("2FA not enabled")?;
        let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, &secret_str).ok_or("Invalid base32 secret")?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
        ).map_err(|e| e.to_string())?;

        let valid = totp.check_current(&command.code).map_err(|e| e.to_string())?;
        
        if valid {
            // 3. 验证成功，发放正式 Token
            let permissions = self.user_repo.get_user_permissions(user.id())
                .await
                .map_err(|e| e.to_string())?;

            let roles_str: Vec<String> = user.roles().iter().map(|r| r.to_string()).collect();
            let perms_str: Vec<String> = permissions.into_iter().map(|p| p.to_string()).collect();

            let (access, refresh, expires) = self.token_service.generate_tokens(
                user.id().to_string(),
                claims.tid,
                roles_str,
                perms_str,
            ).await.map_err(|e| e.to_string())?;

            Ok(Verify2FACodeResponse {
                access_token: access,
                refresh_token: refresh,
                expires_in: expires,
            })
        } else {
            Err("Invalid 2FA code".to_string())
        }
    }
}
