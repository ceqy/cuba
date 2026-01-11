use serde::Deserialize;
use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct CreateBudgetCommand {
    pub wbs_element: String,
    pub fiscal_year: i32,
    pub amount: Decimal,
}

#[derive(Debug, Deserialize)]
pub struct PostDirectCostCommand {
    pub wbs_element: String,
    pub cost_element: String,
    pub amount: Decimal,
    pub posting_date: NaiveDate,
    pub description: Option<String>,
}
