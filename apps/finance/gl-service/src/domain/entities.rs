use cuba_core::Aggregate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: Uuid,
    pub document_number: String,
    pub fiscal_year: i32,
    pub company_code: String,
    pub posting_date: DateTime<Utc>,
    pub status: JournalEntryStatus,
}

impl Aggregate for JournalEntry {
    type Id = Uuid;
    fn id(&self) -> &Self::Id { &self.id }
    fn version(&self) -> u64 { 1 }
    fn take_events(&mut self) -> Vec<Box<dyn cuba_core::DomainEvent>> { vec![] }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum JournalEntryStatus {
    Draft,
    Parked,
    Posted,
    Reversed,
}
