use crate::infrastructure::repository::SettingsRepository;
use anyhow::Result;
use std::sync::Arc;
pub struct SettingsHandler {
    repo: Arc<SettingsRepository>,
}
impl SettingsHandler {
    pub fn new(repo: Arc<SettingsRepository>) -> Self {
        Self { repo }
    }
    pub async fn set(&self, key: String, value: String) -> Result<bool> {
        self.repo.set_setting(&key, &value).await?;
        Ok(true)
    }
}
