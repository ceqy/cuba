//! Enable 2FA Handler

use crate::domain::repositories::{UserRepository, RepositoryError};
use crate::domain::value_objects::UserId;
use std::sync::Arc;
use totp_rs::{Algorithm, TOTP, Secret};
use base32::Alphabet;
use uuid::Uuid;

pub struct Enable2FACommand {
    pub user_id: String,
}

pub struct Enable2FAResponse {
    pub secret_key: String,
    pub qr_code_url: String,
}

pub struct Enable2FAHandler {
    user_repo: Arc<dyn UserRepository>,
}

impl Enable2FAHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn handle(&self, command: Enable2FACommand) -> Result<Enable2FAResponse, String> {
        let user_id = UserId::parse(&command.user_id).map_err(|e| e.to_string())?;
        let mut user = self.user_repo.find_by_id(&user_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("User not found")?;

        // 1. 生成 TOTP 密钥
        let secret_bytes = Uuid::new_v4().as_bytes().to_vec();
        let secret_str = base32::encode(Alphabet::Rfc4648 { padding: false }, &secret_bytes);
        
        // 2. 预存密钥到用户对象（未启用状态）
        user.setup_tfa(secret_str.clone());
        self.user_repo.save(&mut user).await.map_err(|e| e.to_string())?;

        // 3. 生成 QR Code URL (手动构造以避免 API 不兼容)
        let qr_code_url = format!(
            "otpauth://totp/CUBA:{}?secret={}&issuer=CUBA&algorithm=SHA1&digits=6&period=30",
            user.email(),
            secret_str
        );

        Ok(Enable2FAResponse {
            secret_key: secret_str,
            qr_code_url,
        })
    }
}
