use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateRFQCommand {
    pub company_code: String,
    pub purchasing_org: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitQuoteCommand {
    pub rfq_number: String,
    pub supplier_id: String,
}

#[derive(Debug, Deserialize)]
pub struct AwardCommand {
    pub quote_number: String,
}
