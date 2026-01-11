use crate::domain::services::{TokenService, TokenPair, Claims};
use anyhow::{Result, Context};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

pub struct JwtTokenService {
    secret: String,
    access_token_expiry_minutes: i64,
    refresh_token_expiry_days: i64,
}

impl JwtTokenService {
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            access_token_expiry_minutes: 15,
            refresh_token_expiry_days: 7,
        }
    }
}

impl TokenService for JwtTokenService {
    fn generate_tokens(&self, user_id: &str, tenant_id: Option<String>) -> Result<TokenPair> {
        let now = Utc::now();
        let access_expiry = now + Duration::minutes(self.access_token_expiry_minutes);
        let refresh_expiry = now + Duration::days(self.refresh_token_expiry_days);

        let access_claims = Claims {
            sub: user_id.to_string(),
            exp: access_expiry.timestamp() as usize,
            iat: now.timestamp() as usize,
            tenant_id: tenant_id.clone(),
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        ).context("Failed to generate access token")?;

        // Simplified refresh token logic - ideally should be robust and stored securely
        // For now using the same signing key but longer expiry
        let refresh_claims = Claims {
            sub: user_id.to_string(),
            exp: refresh_expiry.timestamp() as usize,
            iat: now.timestamp() as usize,
            tenant_id,
        };

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        ).context("Failed to generate refresh token")?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: (self.access_token_expiry_minutes * 60) as u64,
        })
    }

    fn validate_token(&self, token: &str) -> Result<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        ).context("Failed to validate token")?;

        Ok(token_data.claims)
    }
}
