use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{Rng, thread_rng};
use sha2::{Digest, Sha256};

#[derive(Clone, Default)]
pub struct CryptoService;

impl CryptoService {
    pub fn generate_random_string(&self, len: usize) -> String {
        thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }

    pub fn verify_pkce(&self, code_verifier: &str, code_challenge: &str, method: &str) -> bool {
        match method {
            "plain" => code_verifier == code_challenge,
            "S256" => {
                let mut hasher = Sha256::new();
                hasher.update(code_verifier.as_bytes());
                let result = hasher.finalize();
                let hashed_verifier = URL_SAFE_NO_PAD.encode(result);
                hashed_verifier == code_challenge
            },
            _ => false,
        }
    }
}
