use serde::Deserialize;
use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseOrderCommand {
    pub document_type: i32,
    pub company_code: String,
    pub purchasing_org: String,
    pub purchasing_group: String,
    pub supplier: String,
    pub order_date: Option<NaiveDate>,
    pub currency: String,
    pub items: Vec<CreatePurchaseOrderItemCommand>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePurchaseOrderItemCommand {
    pub item_number: i32,
    pub material: String,
    pub plant: String,
    pub quantity: Decimal,
    pub quantity_unit: String,
    pub net_price: Decimal,
}
