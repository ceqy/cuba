use std::sync::Arc;
use crate::application::commands::{CreateJournalEntryCommand, PostJournalEntryCommand};
use crate::application::queries::{GetJournalEntryQuery, ListJournalEntriesQuery};
use crate::domain::aggregates::journal_entry::{JournalEntry, LineItem, DebitCredit, PostingStatus};
use crate::domain::repositories::JournalRepository;
use cuba_core::domain::Entity;
use uuid::Uuid;
use chrono::Utc;
use rust_decimal::Decimal;

pub struct CreateJournalEntryHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> CreateJournalEntryHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, cmd: CreateJournalEntryCommand) -> Result<JournalEntry, Box<dyn std::error::Error + Send + Sync>> {
        let lines: Vec<LineItem> = cmd.lines.into_iter().enumerate().map(|(i, l)| -> Result<LineItem, Box<dyn std::error::Error + Send + Sync>> {
            Ok(LineItem {
                id: Uuid::new_v4(),
                line_number: (i + 1) as i32,
                account_id: l.account_id,
                debit_credit: match l.debit_credit.as_str() {
                    "S" | "D" => DebitCredit::Debit,
                    "H" | "C" => DebitCredit::Credit,
                    _ => return Err(format!("Invalid debit/credit indicator: {}", l.debit_credit).into()),
                },
                amount: l.amount,
                local_amount: l.amount, // Simplified: assume local currency for now or same amount
                cost_center: l.cost_center,
                profit_center: l.profit_center,
                text: l.text,
            })
        }).collect::<Result<Vec<_>, _>>()?;

        // Create aggregate
        let mut entry = JournalEntry::new(
            cmd.company_code,
            cmd.fiscal_year,
            cmd.posting_date,
            cmd.document_date,
            cmd.currency,
            cmd.reference,
            lines,
            None, // tenant_id
        )?;

        if cmd.post_immediately {
            // In a real app, generate doc number from sequence
            let doc_num = format!("DOC-{}-{}", entry.fiscal_year, Uuid::new_v4().simple().to_string().chars().take(8).collect::<String>());
            entry.post(doc_num)?;
        }

        self.repository.save(&entry).await?;

        Ok(entry)
    }
}

pub struct PostJournalEntryHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> PostJournalEntryHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, cmd: PostJournalEntryCommand) -> Result<JournalEntry, Box<dyn std::error::Error + Send + Sync>> {
        let mut entry = self.repository.find_by_id(&cmd.id).await?
            .ok_or("Journal entry not found")?;

        if entry.status == PostingStatus::Posted {
            return Ok(entry);
        }

        // Generate doc number
        let doc_num = format!("DOC-{}-{}", entry.fiscal_year, Uuid::new_v4().simple().to_string().chars().take(8).collect::<String>());
        entry.post(doc_num)?;

        self.repository.save(&entry).await?;

        Ok(entry)
    }
}

pub struct GetJournalEntryHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> GetJournalEntryHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, query: GetJournalEntryQuery) -> Result<Option<JournalEntry>, Box<dyn std::error::Error + Send + Sync>> {
        self.repository.find_by_id(&query.id).await
    }
}

pub struct ListJournalEntriesHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> ListJournalEntriesHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(&self, query: ListJournalEntriesQuery) -> Result<Vec<JournalEntry>, Box<dyn std::error::Error + Send + Sync>> {
        let status = query.status.as_deref();
        self.repository.search(&query.company_code, status, query.page, query.page_size).await
    }
}
