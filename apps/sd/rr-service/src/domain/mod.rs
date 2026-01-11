use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueContract {
    pub contract_id: Uuid,
    pub contract_number: String,
    pub source_document_number: String,
    pub source_document_type: String,
    pub company_code: String,
    pub customer: String,
    pub created_at: DateTime<Utc>,
    pub obligations: Vec<PerformanceObligation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceObligation {
    pub pob_id: Uuid,
    pub contract_id: Uuid,
    pub pob_code: String,
    pub description: Option<String>,
    pub allocated_price: Option<Decimal>,
    pub currency: String,
    pub recognition_method: String,
    pub recognized_revenue: Decimal,
    pub deferred_revenue: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenuePostingDocument {
    pub posting_id: Uuid,
    pub document_id: String,
    pub posting_date: NaiveDate,
    pub pob_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub posting_type: Option<String>,
    pub accounting_document_number: Option<String>,
    pub created_at: DateTime<Utc>,
}
