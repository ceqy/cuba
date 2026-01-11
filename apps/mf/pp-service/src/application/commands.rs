use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RunMrpCommand {
    pub plant: String,
    pub materials: Vec<String>,
    pub run_type: i32,
    pub planning_mode: String,
}
