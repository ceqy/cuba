//! Event Publisher for GL Service
//!
//! Kafka 事件发布器实现

use async_trait::async_trait;
use serde::Serialize;
use tracing::{info, warn, instrument};
use uuid::Uuid;

use crate::domain::entities::JournalEntry;

// ============================================================================
// Event Types
// ============================================================================

/// 凭证已过账事件
#[derive(Debug, Clone, Serialize)]
pub struct JournalEntryPostedEvent {
    pub event_id: Uuid,
    pub journal_entry_id: String,
    pub company_code: String,
    pub fiscal_year: i32,
    pub document_number: String,
    pub document_type: String,
    pub posting_date: String,
    pub total_debit: String,
    pub total_credit: String,
    pub currency: String,
    pub posted_by: String,
    pub posted_at: String,
}

/// 凭证已冲销事件
#[derive(Debug, Clone, Serialize)]
pub struct JournalEntryReversedEvent {
    pub event_id: Uuid,
    pub original_journal_entry_id: String,
    pub reversal_journal_entry_id: String,
    pub company_code: String,
    pub fiscal_year: i32,
    pub reversal_reason: String,
    pub reversed_by: String,
    pub reversed_at: String,
}

// ============================================================================
// Event Publisher Trait
// ============================================================================

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish_journal_entry_posted(&self, event: JournalEntryPostedEvent) -> anyhow::Result<()>;
    async fn publish_journal_entry_reversed(&self, event: JournalEntryReversedEvent) -> anyhow::Result<()>;
}

// ============================================================================
// Kafka Event Publisher
// ============================================================================

pub struct KafkaEventPublisher {
    // producer: rdkafka::producer::FutureProducer,
    topic_prefix: String,
}

impl KafkaEventPublisher {
    pub fn new(topic_prefix: String) -> Self {
        Self { topic_prefix }
    }

    fn topic_name(&self, event_type: &str) -> String {
        format!("{}.gl.{}", self.topic_prefix, event_type)
    }
}

#[async_trait]
impl EventPublisher for KafkaEventPublisher {
    #[instrument(skip(self, event))]
    async fn publish_journal_entry_posted(&self, event: JournalEntryPostedEvent) -> anyhow::Result<()> {
        let topic = self.topic_name("journal_entry_posted");
        let payload = serde_json::to_string(&event)?;
        
        info!(
            topic = %topic,
            journal_entry_id = %event.journal_entry_id,
            document_number = %event.document_number,
            "Publishing JournalEntryPosted event"
        );

        // TODO: 实际发送到 Kafka
        // self.producer.send(
        //     FutureRecord::to(&topic)
        //         .payload(&payload)
        //         .key(&event.journal_entry_id),
        //     Duration::from_secs(5),
        // ).await?;

        Ok(())
    }

    #[instrument(skip(self, event))]
    async fn publish_journal_entry_reversed(&self, event: JournalEntryReversedEvent) -> anyhow::Result<()> {
        let topic = self.topic_name("journal_entry_reversed");
        let payload = serde_json::to_string(&event)?;
        
        info!(
            topic = %topic,
            original_id = %event.original_journal_entry_id,
            reversal_id = %event.reversal_journal_entry_id,
            "Publishing JournalEntryReversed event"
        );

        // TODO: 实际发送到 Kafka

        Ok(())
    }
}

// ============================================================================
// No-Op Publisher (for testing)
// ============================================================================

pub struct NoOpEventPublisher;

#[async_trait]
impl EventPublisher for NoOpEventPublisher {
    async fn publish_journal_entry_posted(&self, _event: JournalEntryPostedEvent) -> anyhow::Result<()> {
        Ok(())
    }

    async fn publish_journal_entry_reversed(&self, _event: JournalEntryReversedEvent) -> anyhow::Result<()> {
        Ok(())
    }
}

// ============================================================================
// Helper: Convert JournalEntry to Event
// ============================================================================

impl JournalEntryPostedEvent {
    pub fn from_entry(entry: &JournalEntry, posted_by: Uuid) -> Self {
        use chrono::Utc;
        
        Self {
            event_id: Uuid::new_v4(),
            journal_entry_id: entry.id().to_string(),
            company_code: entry.header().company_code.clone(),
            fiscal_year: entry.header().fiscal_period.year(),
            document_number: entry.document_number().map(|d| d.number().to_string()).unwrap_or_default(),
            document_type: entry.header().document_type.clone(),
            posting_date: entry.header().posting_date.to_string(),
            total_debit: entry.total_debit().to_string(),
            total_credit: entry.total_credit().to_string(),
            currency: entry.header().currency.clone(),
            posted_by: posted_by.to_string(),
            posted_at: Utc::now().to_rfc3339(),
        }
    }
}
