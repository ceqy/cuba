use async_trait::async_trait;
use crate::domain::aggregates::journal_entry::JournalEntry;
use uuid::Uuid;
use std::error::Error;

#[async_trait]
pub trait JournalRepository: Send + Sync {
    async fn save(&self, entry: &JournalEntry) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<JournalEntry>, Box<dyn Error + Send + Sync>>;
    async fn search(&self, company_code: &str, status: Option<&str>, page: u64, page_size: u64) -> Result<Vec<JournalEntry>, Box<dyn Error + Send + Sync>>;
    async fn count(&self, company_code: &str, status: Option<&str>) -> Result<i64, Box<dyn Error + Send + Sync>>;
    async fn delete(&self, id: &Uuid) -> Result<(), Box<dyn Error + Send + Sync>>;
}
