use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GenerateForecastCommand {
    pub material: String,
    pub plant: String,
}

#[derive(Debug, Deserialize)]
pub struct TransferCommand {
    pub plan_code: String,
}
