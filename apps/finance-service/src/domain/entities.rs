use cuba_core::Aggregate;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct JournalEntry {
    pub id: Uuid,
    pub company_code: String,
    pub fiscal_year: i32,
    pub document_type: String,
    pub posting_date: DateTime<Utc>,
    pub currency: String,
    pub version: u64,
}

impl Aggregate for JournalEntry {
    type Id = Uuid;
    fn id(&self) -> &Self::Id {
        &self.id
    }
    fn version(&self) -> u64 {
        self.version
    }
    fn take_events(&mut self) -> Vec<Box<dyn cuba_core::DomainEvent>> {
        Vec::new() // 为将来准备
    }
}
