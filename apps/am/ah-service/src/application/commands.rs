use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IngestDataCommand {
    pub equipment_number: String,
    pub sensor_id: String,
    pub value: String,
}
