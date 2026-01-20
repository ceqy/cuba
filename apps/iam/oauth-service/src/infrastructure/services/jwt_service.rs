use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: Option<usize>,
    pub scope: String,
    pub client_id: String,
    pub tenant_id: String,
}

#[derive(Clone)]
pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_access_token(
        &self,
        user_id: &str,
        client_id: &str,
        tenant_id: &str,
        scope: &str,
    ) -> Result<String, anyhow::Error> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as usize;
        let expiration = now + 3600; // 1 hour

        let claims = Claims {
            sub: user_id.to_owned(),
            exp: expiration,
            iat: Some(now),
            scope: scope.to_owned(),
            client_id: client_id.to_owned(),
            tenant_id: tenant_id.to_owned(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )?;
        Ok(token)
    }

    pub fn decode_token(&self, token: &str) -> Result<Claims, anyhow::Error> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    pub fn decode_token_unverified(&self, token: &str) -> Result<Claims, anyhow::Error> {
        let mut validation = Validation::default();
        validation.insecure_disable_signature_validation();
        validation.validate_exp = false;
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )?;
        Ok(token_data.claims)
    }

    pub fn generate_refresh_token(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}
