use std::sync::Arc;
use crate::domain::{Survey, SurveyResponse, Recognition};
use crate::infrastructure::repository::ExperienceRepository;
use crate::application::commands::{LaunchSurveyCommand, SubmitResponseCommand, GiveRecognitionCommand};
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;

pub struct ExperienceHandler {
    repo: Arc<ExperienceRepository>,
}

impl ExperienceHandler {
    pub fn new(repo: Arc<ExperienceRepository>) -> Self {
        Self { repo }
    }

    pub async fn launch_survey(&self, cmd: LaunchSurveyCommand) -> Result<String> {
        let survey_id = Uuid::new_v4();
        let s = Survey {
            survey_id,
            title: cmd.title,
            target_audience: cmd.target_audience,
            status: "ACTIVE".to_string(),
            created_at: Utc::now(),
        };
        self.repo.create_survey(&s).await?;
        Ok(survey_id.to_string())
    }

    pub async fn submit_response(&self, cmd: SubmitResponseCommand) -> Result<()> {
        let r = SurveyResponse {
            response_id: Uuid::new_v4(),
            survey_id: Uuid::parse_str(&cmd.survey_id)?,
            employee_id: cmd.employee_id,
            answers: Some(cmd.answers),
            submitted_at: Utc::now(),
        };
        self.repo.submit_response(&r).await
    }

    pub async fn give_recognition(&self, cmd: GiveRecognitionCommand) -> Result<String> {
        let rec_id = Uuid::new_v4();
        let r = Recognition {
            recognition_id: rec_id,
            giver_employee_id: cmd.giver_employee_id,
            receiver_employee_id: cmd.receiver_employee_id,
            message: Some(cmd.message),
            company_value: cmd.company_value,
            status: "ACTIVE".to_string(),
            created_at: Utc::now(),
        };
        self.repo.create_recognition(&r).await?;
        Ok(rec_id.to_string())
    }
}
