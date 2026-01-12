use bcrypt::{hash, verify, DEFAULT_COST};

#[derive(Clone, Default)]
pub struct BcryptPasswordService;

impl BcryptPasswordService {
    pub fn hash_password(&self, password: &str) -> Result<String, anyhow::Error> {
        Ok(hash(password, DEFAULT_COST)?)
    }

    pub fn verify_password(&self, password: &str, hash_str: &str) -> Result<bool, anyhow::Error> {
        Ok(verify(password, hash_str)?)
    }
}
