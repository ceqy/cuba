use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::{CreateJobApplicationCommand, UpdateStatusCommand, ScheduleInterviewCommand};
use crate::application::handlers::TalentHandler;
use crate::infrastructure::repository::TalentRepository;

use crate::api::proto::hr::ta::v1 as ta_v1;
use crate::api::proto::common::v1 as common_v1;

use ta_v1::talent_acquisition_service_server::TalentAcquisitionService;
use ta_v1::*;

pub struct TaServiceImpl {
    handler: Arc<TalentHandler>,
    repo: Arc<TalentRepository>,
}

impl TaServiceImpl {
    pub fn new(handler: Arc<TalentHandler>, repo: Arc<TalentRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl TalentAcquisitionService for TaServiceImpl {

    async fn create_job_application(
        &self,
        request: Request<CreateJobApplicationRequest>,
    ) -> Result<Response<JobApplicationResponse>, Status> {
        let req = request.into_inner();
        let candidate = req.candidate.unwrap_or_default();
        let cmd = CreateJobApplicationCommand {
            requisition_id: req.requisition_id,
            first_name: candidate.first_name,
            last_name: candidate.last_name,
            email: candidate.email,
            phone: if candidate.phone.is_empty() { None } else { Some(candidate.phone) },
            resume_url: if candidate.resume_url.is_empty() { None } else { Some(candidate.resume_url) },
        };
        let app_id = self.handler.create_application(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(JobApplicationResponse {
            success: true,
            application_id: app_id,
            messages: vec![],
        }))
    }

    async fn get_job_application(
        &self,
        request: Request<GetJobApplicationRequest>,
    ) -> Result<Response<JobApplicationDetail>, Status> {
        let req = request.into_inner();
        let app_id = uuid::Uuid::parse_str(&req.application_id)
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;
        let app = self.repo.find_application_by_id(app_id).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Application not found"))?;
        Ok(Response::new(JobApplicationDetail {
            application_id: app.application_id.to_string(),
            requisition_id: app.requisition_id,
            requisition_title: app.requisition_title.unwrap_or_default(),
            candidate: None, // Simplified
            status: common_v1::ApplicationStatus::New as i32,
            application_date: None,
            attachments: vec![],
            interviews: app.interviews.into_iter().map(|i| InterviewSchedule {
                interview_id: i.interview_id.to_string(),
                interview_type: common_v1::InterviewType::Phone as i32,
                scheduled_time: None,
                interviewer_id: i.interviewer_id.unwrap_or_default(),
                location: i.location.unwrap_or_default(),
                notes: i.notes.unwrap_or_default(),
            }).collect(),
            audit_data: None,
        }))
    }

    async fn update_job_application_status(
        &self,
        request: Request<UpdateJobApplicationStatusRequest>,
    ) -> Result<Response<UpdateJobApplicationStatusResponse>, Status> {
        let req = request.into_inner();
        let status_str = match req.new_status {
            1 => "SUBMITTED",
            2 => "SCREENING",
            3 => "INTERVIEW",
            4 => "OFFER",
            5 => "HIRED",
            6 => "REJECTED",
            _ => "SUBMITTED",
        };
        let cmd = UpdateStatusCommand {
            application_id: req.application_id,
            new_status: status_str.to_string(),
            notes: if req.notes.is_empty() { None } else { Some(req.notes) },
        };
        self.handler.update_status(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(UpdateJobApplicationStatusResponse {
            success: true,
            messages: vec![],
        }))
    }

    async fn schedule_interview(
        &self,
        request: Request<ScheduleInterviewRequest>,
    ) -> Result<Response<ScheduleInterviewResponse>, Status> {
        let req = request.into_inner();
        let interview = req.interview.unwrap_or_default();
        let cmd = ScheduleInterviewCommand {
            application_id: req.application_id,
            interview_type: match interview.interview_type {
                1 => "PHONE".to_string(),
                2 => "VIDEO".to_string(),
                3 => "ONSITE".to_string(),
                _ => "PHONE".to_string(),
            },
            scheduled_time: chrono::Utc::now(), // Simplified
            interviewer_id: if interview.interviewer_id.is_empty() { None } else { Some(interview.interviewer_id) },
            location: if interview.location.is_empty() { None } else { Some(interview.location) },
        };
        let interview_id = self.handler.schedule_interview(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ScheduleInterviewResponse {
            success: true,
            interview_id,
            messages: vec![],
        }))
    }

    // Stubs
    async fn submit_resume(&self, _r: Request<SubmitResumeRequest>) -> Result<Response<JobApplicationResponse>, Status> { Err(Status::unimplemented("")) }
    async fn list_job_applications(&self, _r: Request<ListJobApplicationsRequest>) -> Result<Response<ListJobApplicationsResponse>, Status> { Err(Status::unimplemented("")) }
}
