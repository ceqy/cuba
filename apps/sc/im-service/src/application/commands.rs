use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct PostStockMovementCommand {
    pub posting_date: NaiveDate,
    pub document_date: NaiveDate,
    pub header_text: Option<String>,
    pub reference_document: Option<String>,
    
    pub items: Vec<StockMovementItemCommand>,
}

#[derive(Debug, Deserialize)]
pub struct StockMovementItemCommand {
    pub movement_type: String, // 101, 201
    pub material: String,
    pub plant: String,
    pub storage_location: String,
    pub batch: Option<String>,
    pub quantity: Decimal,
    pub unit_of_measure: String,
    // For MVP we derive S/H from Movement Type logic in Handler
}

#[derive(Debug, Deserialize)]
pub struct GetStockOverviewQuery {
    pub material: String,
    pub plant: String,
    pub storage_location: Option<String>,
}
