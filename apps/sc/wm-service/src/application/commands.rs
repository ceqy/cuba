use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateTOCommand {
    pub warehouse_number: String,
    pub movement_type: String,
    pub reference_doc_number: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmTOCommand {
    pub warehouse_number: String,
    pub to_number: String,
}
