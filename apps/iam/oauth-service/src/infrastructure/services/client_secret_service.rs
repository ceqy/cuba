use bcrypt::{DEFAULT_COST, hash, verify};

#[derive(Clone, Default)]
pub struct ClientSecretService;

impl ClientSecretService {
    pub fn hash_secret(&self, secret: &str) -> Result<String, anyhow::Error> {
        Ok(hash(secret, DEFAULT_COST)?)
    }

    pub fn verify_secret(&self, secret: &str, hashed_secret: &str) -> Result<bool, anyhow::Error> {
        Ok(verify(secret, hashed_secret)?)
    }
}
