use sqlx::PgPool; use crate::domain::SystemSetting; use anyhow::Result;
pub struct SettingsRepository { pool: PgPool }
impl SettingsRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn get_setting(&self, key: &str) -> Result<Option<SystemSetting>> {
        let r = sqlx::query!("SELECT * FROM system_settings WHERE setting_key = $1", key).fetch_optional(&self.pool).await?;
        Ok(r.map(|r| SystemSetting { setting_id: r.setting_id, setting_key: r.setting_key, setting_value: r.setting_value, description: r.description, updated_at: r.updated_at }))
    }
    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query!("INSERT INTO system_settings (setting_id, setting_key, setting_value) VALUES (uuid_generate_v4(), $1, $2) ON CONFLICT (setting_key) DO UPDATE SET setting_value = EXCLUDED.setting_value, updated_at = NOW()", key, value).execute(&self.pool).await?;
        Ok(())
    }
}
