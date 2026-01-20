use crate::application::commands::{
    GiveRecognitionCommand, LaunchSurveyCommand, SubmitResponseCommand,
};
use crate::application::handlers::ExperienceHandler;
use crate::infrastructure::repository::ExperienceRepository;
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::api::proto::common::v1 as common_v1;
use crate::api::proto::hr::ex::v1 as ex_v1;

use ex_v1::employee_experience_service_server::EmployeeExperienceService;
use ex_v1::*;

pub struct ExServiceImpl {
    handler: Arc<ExperienceHandler>,
    repo: Arc<ExperienceRepository>,
}

impl ExServiceImpl {
    pub fn new(handler: Arc<ExperienceHandler>, repo: Arc<ExperienceRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl EmployeeExperienceService for ExServiceImpl {
    async fn launch_survey(
        &self,
        request: Request<LaunchSurveyRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        let survey = req.survey.unwrap_or_default();
        let cmd = LaunchSurveyCommand {
            title: survey.title,
            target_audience: if survey.target_audience.is_empty() {
                None
            } else {
                Some(survey.target_audience)
            },
        };
        let survey_id = self
            .handler
            .launch_survey(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_v1::JobInfo {
            job_id: survey_id,
            job_type: "SURVEY_LAUNCH".to_string(),
            status: common_v1::JobStatus::Completed as i32,
            progress_percentage: 100,
            messages: vec![],
            error_detail: "".to_string(),
            created_at: None,
            started_at: None,
            completed_at: None,
        }))
    }

    async fn submit_survey_response(
        &self,
        request: Request<SubmitSurveyResponseRequest>,
    ) -> Result<Response<SubmitSurveyResponseResponse>, Status> {
        let req = request.into_inner();
        let answers: serde_json::Value = req
            .answers
            .iter()
            .map(|a| serde_json::json!({ "question_id": a.question_id, "value": a.answer_value }))
            .collect();
        let cmd = SubmitResponseCommand {
            survey_id: req.survey_id,
            employee_id: req.respondent_employee_id,
            answers,
        };
        self.handler
            .submit_response(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(SubmitSurveyResponseResponse {
            success: true,
            messages: vec![],
        }))
    }

    async fn give_recognition(
        &self,
        request: Request<GiveRecognitionRequest>,
    ) -> Result<Response<GiveRecognitionResponse>, Status> {
        let req = request.into_inner();
        let rec = req.recognition.unwrap_or_default();
        let cmd = GiveRecognitionCommand {
            giver_employee_id: rec.giver_employee_id,
            receiver_employee_id: rec.receiver_employee_id,
            message: rec.message,
            company_value: if rec.company_value_aligned.is_empty() {
                None
            } else {
                Some(rec.company_value_aligned)
            },
        };
        let rec_id = self
            .handler
            .give_recognition(cmd)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GiveRecognitionResponse {
            success: true,
            recognition_id: rec_id,
            messages: vec![],
        }))
    }

    async fn list_recognitions(
        &self,
        request: Request<ListRecognitionsRequest>,
    ) -> Result<Response<ListRecognitionsResponse>, Status> {
        let req = request.into_inner();
        let recs = self
            .repo
            .list_recognitions(&req.employee_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListRecognitionsResponse {
            recognitions: recs
                .into_iter()
                .map(|r| Recognition {
                    recognition_id: r.recognition_id.to_string(),
                    giver_employee_id: r.giver_employee_id,
                    receiver_employee_id: r.receiver_employee_id,
                    message: r.message.unwrap_or_default(),
                    company_value_aligned: r.company_value.unwrap_or_default(),
                    recognition_time: None,
                    status: common_v1::RecognitionStatus::Approved as i32,
                })
                .collect(),
            pagination: None,
        }))
    }

    // Stubs
    async fn list_surveys(
        &self,
        _r: Request<ListSurveysRequest>,
    ) -> Result<Response<ListSurveysResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn get_survey_analytics(
        &self,
        _r: Request<GetSurveyAnalyticsRequest>,
    ) -> Result<Response<SurveyAnalytics>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn get_recognition_feed(
        &self,
        _r: Request<GetRecognitionFeedRequest>,
    ) -> Result<Response<GetRecognitionFeedResponse>, Status> {
        Err(Status::unimplemented(""))
    }
}
