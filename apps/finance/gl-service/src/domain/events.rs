//! Domain Events for GL Service
//!
//! 领域事件定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::JournalEntryId;

/// 领域事件 trait
pub trait DomainEvent: Send + Sync {
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> Uuid;
    fn occurred_at(&self) -> DateTime<Utc>;
}

// ============================================================================
// Journal Entry Events
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryCreated {
    pub journal_entry_id: Uuid,
    pub company_code: String,
    pub document_number: String,
    pub fiscal_year: i32,
    pub created_by: Uuid,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for JournalEntryCreated {
    fn event_type(&self) -> &'static str { "JournalEntryCreated" }
    fn aggregate_id(&self) -> Uuid { self.journal_entry_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryPosted {
    pub journal_entry_id: Uuid,
    pub company_code: String,
    pub document_number: String,
    pub fiscal_year: i32,
    pub posted_by: Uuid,
    pub posting_date: DateTime<Utc>,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for JournalEntryPosted {
    fn event_type(&self) -> &'static str { "JournalEntryPosted" }
    fn aggregate_id(&self) -> Uuid { self.journal_entry_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryReversed {
    pub journal_entry_id: Uuid,
    pub reversal_document_id: Uuid,
    pub reversal_reason: String,
    pub reversed_by: Uuid,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for JournalEntryReversed {
    fn event_type(&self) -> &'static str { "JournalEntryReversed" }
    fn aggregate_id(&self) -> Uuid { self.journal_entry_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearingCompleted {
    pub clearing_document_id: Uuid,
    pub cleared_items: Vec<Uuid>,
    pub clearing_date: DateTime<Utc>,
    pub cleared_by: Uuid,
    pub occurred_at: DateTime<Utc>,
}

impl DomainEvent for ClearingCompleted {
    fn event_type(&self) -> &'static str { "ClearingCompleted" }
    fn aggregate_id(&self) -> Uuid { self.clearing_document_id }
    fn occurred_at(&self) -> DateTime<Utc> { self.occurred_at }
}

/// 事件容器，用于存储聚合根产生的所有事件
#[derive(Default)]
pub struct DomainEvents {
    events: Vec<Box<dyn DomainEvent>>,
}

impl DomainEvents {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }
    
    pub fn push<E: DomainEvent + 'static>(&mut self, event: E) {
        self.events.push(Box::new(event));
    }
    
    pub fn take(&mut self) -> Vec<Box<dyn DomainEvent>> {
        std::mem::take(&mut self.events)
    }
    
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
