use crate::domain::entities::JournalEntry;
use crate::domain::repository::JournalEntryRepository;
use async_trait::async_trait;
use cuba_core::Repository;
use sqlx::PgPool;
use std::sync::Arc;

pub struct PgJournalEntryRepository {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl PgJournalEntryRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<JournalEntry> for PgJournalEntryRepository {
    async fn save(&self, _aggregate: &mut JournalEntry) -> anyhow::Result<()> {
        Ok(())
    }

    async fn find_by_id(&self, _id: &uuid::Uuid) -> anyhow::Result<Option<JournalEntry>> {
        Ok(None)
    }
}

#[async_trait]
impl JournalEntryRepository for PgJournalEntryRepository {}
