use serde::Deserialize;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct CreateBatchCommand {
    pub material: String,
    pub plant: String,
    pub production_date: Option<NaiveDate>,
    pub expiration_date: Option<NaiveDate>,
    pub supplier_batch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TraceCommand {
    pub material: String,
    pub batch: String,
    pub plant: String,
    pub depth_limit: i32,
}
