use rust_decimal::Decimal;
use chrono::{NaiveDate, DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialStock {
    pub stock_id: Uuid,
    pub plant: String,
    pub storage_location: String,
    pub material: String,
    pub batch: String,
    
    pub unrestricted_quantity: Decimal,
    pub quality_inspection_quantity: Decimal,
    pub blocked_quantity: Decimal,
    
    pub base_unit: String,
    
    pub last_movement_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDocument {
    pub document_id: Uuid,
    pub document_number: String,
    pub fiscal_year: i32,
    pub document_date: NaiveDate,
    pub posting_date: NaiveDate,
    pub document_type: Option<String>,
    pub reference_document: Option<String>,
    pub header_text: Option<String>,
    
    pub items: Vec<MaterialDocumentItem>,
    
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialDocumentItem {
    pub item_id: Uuid,
    pub document_id: Uuid,
    pub line_item_number: i32,
    pub movement_type: String,
    pub debit_credit_indicator: String, // S or H
    pub material: String,
    pub plant: String,
    pub storage_location: String,
    pub batch: Option<String>,
    pub quantity: Decimal,
    pub unit_of_measure: String,
    pub amount_lc: Option<Decimal>,
}
