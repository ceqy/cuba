use rust_decimal::Decimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionLot {
    pub lot_id: Uuid,
    pub inspection_lot_number: String,
    pub material: String,
    pub plant: String,
    pub lot_quantity: Decimal,
    pub quantity_unit: String,
    pub origin: String,
    pub creation_date: DateTime<Utc>,
    
    pub ud_code: Option<String>,
    pub ud_date: Option<DateTime<Utc>>,
    pub ud_note: Option<String>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Aggregates
    pub characteristics: Vec<InspectionCharacteristic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionCharacteristic {
    pub char_id: Uuid,
    pub lot_id: Uuid,
    pub characteristic_number: String,
    pub description: Option<String>,
    pub inspection_method: Option<String>,
    pub result_value: Option<String>,
    pub result_status: String,
}
