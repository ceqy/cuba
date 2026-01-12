use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Contract {
    pub contract_id: Uuid,
    pub contract_number: String,
    pub company_code: String,
    pub supplier: String,
    pub purchasing_org: String,
    pub purchasing_group: Option<String>,
    pub validity_start: Option<NaiveDate>,
    pub validity_end: Option<NaiveDate>,
    pub target_value: Option<Decimal>,
    pub currency: String,
    pub release_status: String,
    pub created_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub items: Vec<ContractItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ContractItem {
    pub item_id: Uuid,
    pub contract_id: Uuid,
    pub item_number: i32,
    pub material: Option<String>,
    pub short_text: Option<String>,
    pub target_quantity: Option<Decimal>,
    pub unit: String,
    pub net_price: Option<Decimal>,
    pub price_currency: String,
    pub plant: Option<String>,
}
