use crate::domain::entities::{AuthorizationCode, RefreshToken};
use crate::domain::repositories::{AuthCodeRepository, ClientRepository, RefreshTokenRepository};
use crate::infrastructure::services::{ClientSecretService, CryptoService, JwtService};
use chrono::{Duration, Utc};
use std::sync::Arc;

pub struct AuthorizeHandler<C: ClientRepository, A: AuthCodeRepository> {
    client_repo: Arc<C>,
    auth_code_repo: Arc<A>,
    crypto_service: Arc<CryptoService>,
}

impl<C: ClientRepository, A: AuthCodeRepository> AuthorizeHandler<C, A> {
    pub fn new(
        client_repo: Arc<C>,
        auth_code_repo: Arc<A>,
        crypto_service: Arc<CryptoService>,
    ) -> Self {
        Self {
            client_repo,
            auth_code_repo,
            crypto_service,
        }
    }

    pub async fn handle(
        &self,
        client_id: String,
        user_id: String,
        redirect_uri: String,
        scope: String,
        code_challenge: Option<String>,
        code_challenge_method: Option<String>,
    ) -> Result<String, anyhow::Error> {
        // Validate Client
        let client = self
            .client_repo
            .find_by_id(&client_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        if !client.redirect_uris.contains(&redirect_uri) {
            return Err(anyhow::anyhow!("Invalid redirect URI"));
        }

        // Generate Code
        let code = self.crypto_service.generate_random_string(32);
        let auth_code = AuthorizationCode {
            code: code.clone(),
            client_id,
            user_id,
            redirect_uri,
            scope,
            code_challenge,
            code_challenge_method,
            expires_at: Utc::now() + Duration::minutes(10),
            created_at: Utc::now(),
        };

        self.auth_code_repo.save(&auth_code).await?;
        Ok(code)
    }
}

pub struct TokenHandler<C: ClientRepository, A: AuthCodeRepository, R: RefreshTokenRepository> {
    client_repo: Arc<C>,
    auth_code_repo: Arc<A>,
    refresh_token_repo: Arc<R>,
    jwt_service: Arc<JwtService>,
    crypto_service: Arc<CryptoService>,
    secret_service: Arc<ClientSecretService>,
}

impl<C: ClientRepository, A: AuthCodeRepository, R: RefreshTokenRepository> TokenHandler<C, A, R> {
    pub fn new(
        client_repo: Arc<C>,
        auth_code_repo: Arc<A>,
        refresh_token_repo: Arc<R>,
        jwt_service: Arc<JwtService>,
        crypto_service: Arc<CryptoService>,
        secret_service: Arc<ClientSecretService>,
    ) -> Self {
        Self {
            client_repo,
            auth_code_repo,
            refresh_token_repo,
            jwt_service,
            crypto_service,
            secret_service,
        }
    }

    pub async fn handle_auth_code(
        &self,
        code: String,
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        code_verifier: Option<String>,
    ) -> Result<(String, String, i32), anyhow::Error> {
        // Find and Validate Code
        let auth_code = self
            .auth_code_repo
            .find_by_code(&code)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired code"))?;

        if auth_code.client_id != client_id {
            return Err(anyhow::anyhow!("Client ID mismatch"));
        }

        if auth_code.redirect_uri != redirect_uri {
            return Err(anyhow::anyhow!("Redirect URI mismatch"));
        }

        // Validate Client Secret
        let client = self
            .client_repo
            .find_by_id(&client_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        if !self
            .secret_service
            .verify_secret(&client_secret, &client.client_secret)?
        {
            return Err(anyhow::anyhow!("Invalid client secret"));
        }

        // PKCE Verification if applicable
        if let Some(challenge) = auth_code.code_challenge {
            let verifier =
                code_verifier.ok_or_else(|| anyhow::anyhow!("Code verifier required for PKCE"))?;
            let method = auth_code
                .code_challenge_method
                .unwrap_or_else(|| "plain".to_string());
            if !self
                .crypto_service
                .verify_pkce(&verifier, &challenge, &method)
            {
                return Err(anyhow::anyhow!("PKCE verification failed"));
            }
        }

        // Consume Code
        self.auth_code_repo.delete(&code).await?;

        // Generate Tokens
        let access_token = self.jwt_service.generate_access_token(
            &auth_code.user_id,
            &client_id,
            "default",
            &auth_code.scope,
        )?;
        let refresh_token_val = self.jwt_service.generate_refresh_token();

        let refresh_token = RefreshToken {
            token: refresh_token_val.clone(),
            client_id,
            user_id: auth_code.user_id,
            scope: auth_code.scope,
            expires_at: Utc::now() + Duration::days(30),
            created_at: Utc::now(),
        };

        self.refresh_token_repo.save(&refresh_token).await?;

        Ok((access_token, refresh_token_val, 3600))
    }

    pub async fn handle_client_credentials(
        &self,
        client_id: String,
        client_secret: String,
        scope: String,
    ) -> Result<(String, i32), anyhow::Error> {
        let client = self
            .client_repo
            .find_by_id(&client_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        if !self
            .secret_service
            .verify_secret(&client_secret, &client.client_secret)?
        {
            return Err(anyhow::anyhow!("Invalid client secret"));
        }

        // System user ID for client credentials
        let access_token = self.jwt_service.generate_access_token(
            &format!("client:{}", client_id),
            &client_id,
            "default",
            &scope,
        )?;

        Ok((access_token, 3600))
    }

    pub async fn handle_refresh_token(
        &self,
        token: String,
        client_id: String,
        client_secret: String,
    ) -> Result<(String, String, i32), anyhow::Error> {
        let rt = self
            .refresh_token_repo
            .find_by_token(&token)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired refresh token"))?;

        if rt.client_id != client_id {
            return Err(anyhow::anyhow!("Client ID mismatch"));
        }

        let client = self
            .client_repo
            .find_by_id(&client_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

        if !self
            .secret_service
            .verify_secret(&client_secret, &client.client_secret)?
        {
            return Err(anyhow::anyhow!("Invalid client secret"));
        }

        // Rotate Refresh Token (optional but safer)
        self.refresh_token_repo.delete(&token).await?;

        let access_token = self.jwt_service.generate_access_token(
            &rt.user_id,
            &client_id,
            "default",
            &rt.scope,
        )?;
        let new_refresh_token_val = self.jwt_service.generate_refresh_token();

        let new_rt = RefreshToken {
            token: new_refresh_token_val.clone(),
            client_id,
            user_id: rt.user_id,
            scope: rt.scope,
            expires_at: Utc::now() + Duration::days(30),
            created_at: Utc::now(),
        };

        self.refresh_token_repo.save(&new_rt).await?;

        Ok((access_token, new_refresh_token_val, 3600))
    }
}
