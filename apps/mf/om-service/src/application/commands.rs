use serde::Deserialize;
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct CreateOrderCommand {
    pub supplier: String,
    pub company_code: String,
    pub purchasing_org: Option<String>,
    pub items: Vec<OrderItemInput>,
}

#[derive(Debug, Deserialize)]
pub struct OrderItemInput {
    pub material: String,
    pub quantity: Decimal,
    pub plant: Option<String>,
    pub components: Vec<ComponentInput>,
}

#[derive(Debug, Deserialize)]
pub struct ComponentInput {
    pub material: String,
    pub quantity: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct PostComponentsCommand {
    pub po_number: String,
}

#[derive(Debug, Deserialize)]
pub struct ReceiveGoodsCommand {
    pub po_number: String,
}
