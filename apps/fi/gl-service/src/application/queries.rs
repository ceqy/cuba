use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct GetJournalEntryQuery {
    pub id: Uuid,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListJournalEntriesQuery {
    pub company_code: String,
    pub status: Option<String>,
    pub page: u64,
    pub page_size: u64,
}
