use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateContractCommand {
    pub source_document_number: String,
    pub source_document_type: String,
    pub company_code: String,
    pub customer: String,
}

#[derive(Debug, Deserialize)]
pub struct RunPostingCommand {
    pub company_code: String,
    pub posting_period: String,
}

#[derive(Debug, Deserialize)]
pub struct POBInput {
    pub description: String,
    pub allocated_price: Decimal,
}
