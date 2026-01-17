use serde::Deserialize;
use rust_decimal::Decimal;
use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct LineItemDTO {
    pub account_id: String,
    pub debit_credit: String, // "D" or "C"
    pub amount: Decimal,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateJournalEntryCommand {
    pub company_code: String,
    pub fiscal_year: i32,
    pub posting_date: NaiveDate,
    pub document_date: NaiveDate,
    pub currency: String,
    pub reference: Option<String>,
    pub lines: Vec<LineItemDTO>,
    pub post_immediately: bool,
}

#[derive(Debug, Deserialize)]
pub struct PostJournalEntryCommand {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ReverseJournalEntryCommand {
    pub id: Uuid,
    pub reversal_reason: String,
    pub posting_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct ParkJournalEntryCommand {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJournalEntryCommand {
    pub id: Uuid,
    pub posting_date: Option<NaiveDate>,
    pub document_date: Option<NaiveDate>,
    pub reference: Option<String>,
    pub lines: Option<Vec<LineItemDTO>>,
}
