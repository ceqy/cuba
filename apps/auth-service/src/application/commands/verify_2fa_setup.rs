//! Verify 2FA Setup Handler

use crate::domain::repositories::{UserRepository, RepositoryError};
use crate::domain::value_objects::UserId;
use std::sync::Arc;
use totp_rs::{Algorithm, TOTP};
use base32::Alphabet;

pub struct Verify2FASetupCommand {
    pub user_id: String,
    pub code: String,
}

pub struct Verify2FASetupHandler {
    user_repo: Arc<dyn UserRepository>,
}

impl Verify2FASetupHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn handle(&self, command: Verify2FASetupCommand) -> Result<(), String> {
        let user_id = UserId::parse(&command.user_id).map_err(|e| e.to_string())?;
        let mut user = self.user_repo.find_by_id(&user_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("User not found")?;

        let secret_str = user.tfa_secret().as_ref().ok_or("2FA not set up for user")?.clone();
        let secret_bytes = base32::decode(Alphabet::Rfc4648 { padding: false }, secret_str).ok_or("Invalid base32 secret")?;
        
        // TOTP::new(algo, digits, skew, step, secret)
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            secret_bytes,
        ).map_err(|e| e.to_string())?;

        // check(code, timestamp) - use system time
        let valid = totp.check_current(&command.code).map_err(|e| e.to_string())?;
        
        if valid {
            // 验证成功，正式启用
            // 生成一些恢复码
            let recovery_codes = vec!["RECOVERY1".to_string(), "RECOVERY2".to_string()]; // FIXME: Generate properly
            user.enable_tfa(recovery_codes);
            self.user_repo.save(&mut user).await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Invalid 2FA code".to_string())
        }
    }
}
