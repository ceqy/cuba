use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectBudget {
    pub budget_id: Uuid,
    pub wbs_element: String,
    pub fiscal_year: i32,
    pub budget_amount: Decimal,
    pub currency: String,
    pub version: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostPosting {
    pub posting_id: Uuid,
    pub wbs_element: String,
    pub cost_element: String,
    pub cost_element_type: String,
    pub amount: Decimal,
    pub currency: String,
    pub posting_date: NaiveDate,
    pub description: Option<String>,
    pub document_number: Option<String>,
}
