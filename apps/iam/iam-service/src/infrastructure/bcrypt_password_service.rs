use async_trait::async_trait;
use crate::domain::services::PasswordService;
use anyhow::{Result, Context};

pub struct BcryptPasswordService {
    cost: u32,
}

impl BcryptPasswordService {
    pub fn new(cost: u32) -> Self {
        Self { cost }
    }
}

impl Default for BcryptPasswordService {
    fn default() -> Self {
        Self { cost: bcrypt::DEFAULT_COST }
    }
}

#[async_trait]
impl PasswordService for BcryptPasswordService {
    async fn hash(&self, plain_password: &str) -> Result<String> {
        let hash = bcrypt::hash(plain_password, self.cost)
            .context("Failed to hash password")?;
        Ok(hash)
    }

    async fn verify(&self, plain_password: &str, hashed_password: &str) -> Result<bool> {
        let matches = bcrypt::verify(plain_password, hashed_password)
            .context("Failed to verify password")?;
        Ok(matches)
    }
}
