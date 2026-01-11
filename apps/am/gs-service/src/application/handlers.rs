use std::sync::Arc; use crate::infrastructure::repository::SettingsRepository; use anyhow::Result;
pub struct SettingsHandler { repo: Arc<SettingsRepository> }
impl SettingsHandler {
    pub fn new(repo: Arc<SettingsRepository>) -> Self { Self { repo } }
    pub async fn set(&self, key: String, value: String) -> Result<bool> { self.repo.set_setting(&key, &value).await?; Ok(true) }
}
