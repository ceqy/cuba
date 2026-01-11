use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankStatement {
    pub statement_id: Uuid,
    pub company_code: String,
    pub statement_format: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub transactions: Vec<StatementTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementTransaction {
    pub transaction_id: Uuid,
    pub statement_id: Uuid,
    pub value_date: NaiveDate,
    pub amount: Decimal,
    pub currency: String,
    pub memo: Option<String>,
    pub partner_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRun {
    pub run_id: Uuid,
    pub run_number: String,
    pub company_codes: Option<String>,
    pub posting_date: Option<NaiveDate>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub documents: Vec<PaymentDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentDocument {
    pub doc_id: Uuid,
    pub run_id: Uuid,
    pub document_number: String,
    pub fiscal_year: Option<i32>,
    pub amount: Decimal,
    pub currency: String,
    pub payee_name: Option<String>,
}
