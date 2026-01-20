use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SyncBOMCommand {
    pub material: String,
    pub plant: String,
    pub bom_usage: String,
    pub base_quantity: Decimal,
    pub items: Vec<BOMItemCmd>,
}

#[derive(Debug, Deserialize)]
pub struct BOMItemCmd {
    pub item_node: String,
    pub component_material: String,
    pub component_quantity: Decimal,
}
