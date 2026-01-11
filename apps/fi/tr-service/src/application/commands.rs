use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProcessStatementCommand {
    pub company_code: String,
}

#[derive(Debug, Deserialize)]
pub struct ExecutePaymentRunCommand {
    pub run_id: String,
    pub company_codes: Vec<String>,
}
