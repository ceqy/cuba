use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub roles: Vec<String>,
    pub tenant_id: String,
}

#[derive(Clone)]
pub struct JwtTokenService {
    secret: String,
}

impl JwtTokenService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_token(&self, user_id: &str, tenant_id: &str, roles: Vec<String>) -> Result<String, anyhow::Error> {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as usize + 3600 * 24; // 24 hours

        let claims = Claims {
            sub: user_id.to_owned(),
            exp: expiration,
            roles,
            tenant_id: tenant_id.to_owned(),
        };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_bytes()))?;
        Ok(token)
    }
}
