use super::entities::JournalEntry;
use cuba_core::Repository;
use async_trait::async_trait;
use uuid::Uuid;
use anyhow::Result;

#[async_trait]
pub trait JournalEntryRepository: Repository<JournalEntry> {
    async fn find_by_document_number(&self, company_code: &str, doc_num: &str, fiscal_year: i32) -> Result<Option<JournalEntry>>;
}
