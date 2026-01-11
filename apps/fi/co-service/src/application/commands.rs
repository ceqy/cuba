use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ExecuteAllocationCommand {
    pub controlling_area: String,
    pub fiscal_year: i32,
    pub fiscal_period: i32,
    pub allocation_cycle: String,
    pub allocation_type: String,
    pub test_run: bool,
}
