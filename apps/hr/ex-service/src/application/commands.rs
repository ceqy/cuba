use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LaunchSurveyCommand {
    pub title: String,
    pub target_audience: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitResponseCommand {
    pub survey_id: String,
    pub employee_id: String,
    pub answers: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct GiveRecognitionCommand {
    pub giver_employee_id: String,
    pub receiver_employee_id: String,
    pub message: String,
    pub company_value: Option<String>,
}
