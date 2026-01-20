use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AnalyzeSalesCommand {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub top_n: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetTrendCommand {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}
