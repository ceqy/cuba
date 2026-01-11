use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct CreateSalesOrderCommand {
    pub order_type: String, // OR
    pub sales_org: String,
    pub distribution_channel: String,
    pub division: String,
    
    pub sold_to_party: String,
    pub ship_to_party: Option<String>,
    pub customer_po: Option<String>,
    pub customer_po_date: Option<NaiveDate>,
    
    pub requested_delivery_date: Option<NaiveDate>,
    pub currency: String,
    
    pub items: Vec<CreateSalesOrderItemCommand>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSalesOrderItemCommand {
    pub item_number: i32,
    pub material: String,
    pub order_quantity: Decimal,
    pub sales_unit: String,
    pub plant: Option<String>,
    pub storage_location: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetSalesOrderQuery {
    pub order_number: String,
}

#[derive(Debug, Deserialize)]
pub struct ListSalesOrdersQuery {
    pub sold_to_party: Option<String>,
    pub limit: i32,
}
