use crate::domain::entities::JournalEntry;
use async_trait::async_trait;
use cuba_core::Repository;

#[async_trait]
pub trait JournalEntryRepository: Repository<JournalEntry> {
    // Add domain-specific repository methods here
}
