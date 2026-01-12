use sqlx::PgPool; use crate::domain::SystemSetting; use anyhow::Result;
pub struct SettingsRepository { pool: PgPool }
impl SettingsRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn get_setting(&self, key: &str) -> Result<Option<SystemSetting>> {
        let r = sqlx::query_as::<_, SystemSetting>("SELECT setting_id, setting_key, setting_value, description, updated_at FROM system_settings WHERE setting_key = $1")
            .bind(key)
            .fetch_optional(&self.pool).await?;
        Ok(r)
    }
    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query("INSERT INTO system_settings (setting_id, setting_key, setting_value) VALUES (gen_random_uuid(), $1, $2) ON CONFLICT (setting_key) DO UPDATE SET setting_value = EXCLUDED.setting_value, updated_at = NOW()")
            .bind(key)
            .bind(value)
        .execute(&self.pool).await?;
        Ok(())
    }
}
