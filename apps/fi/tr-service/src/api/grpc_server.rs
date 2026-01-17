use tonic::{Request, Response, Status};
use std::sync::Arc;
use crate::application::commands::{ProcessStatementCommand, ExecutePaymentRunCommand};
use crate::application::handlers::TreasuryHandler;
use crate::infrastructure::repository::TreasuryRepository;

use crate::api::proto::fi::tr::v1 as tr_v1;
use crate::api::proto::common::v1 as common_v1;

use tr_v1::treasury_service_server::TreasuryService;
use tr_v1::*;

pub struct TrServiceImpl {
    handler: Arc<TreasuryHandler>,
    repo: Arc<TreasuryRepository>,
}

impl TrServiceImpl {
    pub fn new(handler: Arc<TreasuryHandler>, repo: Arc<TreasuryRepository>) -> Self {
        Self { handler, repo }
    }
}

#[tonic::async_trait]
impl TreasuryService for TrServiceImpl {

    async fn process_bank_statement(
        &self,
        request: Request<ProcessBankStatementRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        let cmd = ProcessStatementCommand { company_code: req.company_code };
        let stmt_id = self.handler.process_statement(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_v1::JobInfo {
            job_id: stmt_id,
            job_type: "BANK_STATEMENT".to_string(),
            status: common_v1::JobStatus::Completed as i32,
            progress_percentage: 100,
            messages: vec![],
            error_detail: "".to_string(),
            created_at: None,
            started_at: None,
            completed_at: None,
        }))
    }

    async fn get_bank_statement_detail(
        &self,
        request: Request<GetBankStatementRequest>,
    ) -> Result<Response<BankStatementDetail>, Status> {
        let req = request.into_inner();
        let id = uuid::Uuid::parse_str(&req.statement_id)
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;
        let stmt = self.repo.find_statement_by_id(id).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Statement not found"))?;
        Ok(Response::new(BankStatementDetail {
            statement_id: stmt.statement_id.to_string(),
            company_code: stmt.company_code,
            format: common_v1::BankStatementFormat::Mt940 as i32,
            transactions: stmt.transactions.into_iter().map(|t| BankStatementTransaction {
                transaction_id: t.transaction_id.to_string(),
                value_date: None,
                amount: Some(common_v1::MonetaryValue { value: t.amount.to_string(), currency_code: t.currency }),
                memo: t.memo.unwrap_or_default(),
                partner_name: t.partner_name.unwrap_or_default(),
            }).collect(),
            audit_data: None,
        }))
    }

    async fn execute_payment_run(
        &self,
        request: Request<ExecutePaymentRunRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        let params = req.parameters.unwrap_or_default();
        let cmd = ExecutePaymentRunCommand {
            run_id: req.run_id,
            company_codes: params.company_codes,
        };
        let run_id = self.handler.execute_payment_run(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_v1::JobInfo {
            job_id: run_id,
            job_type: "PAYMENT_RUN".to_string(),
            status: common_v1::JobStatus::Completed as i32,
            progress_percentage: 100,
            messages: vec![],
            error_detail: "".to_string(),
            created_at: None,
            started_at: None,
            completed_at: None,
        }))
    }

    async fn get_payment_run_status(
        &self,
        request: Request<GetPaymentRunRequest>,
    ) -> Result<Response<PaymentRunDetail>, Status> {
        let req = request.into_inner();
        let id = uuid::Uuid::parse_str(&req.run_id)
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;
        let run = self.repo.find_run_by_id(id).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Run not found"))?;
        Ok(Response::new(PaymentRunDetail {
            run_id: run.run_id.to_string(),
            status: common_v1::JobStatus::Completed as i32,
            paid_documents: run.documents.into_iter().map(|d| PaymentDocument {
                document_reference: Some(common_v1::SystemDocumentReference {
                    document_number: d.document_number,
                    fiscal_year: d.fiscal_year.unwrap_or(2026),
                    company_code: "".to_string(),
                    document_category: "".to_string(),
                    document_type: "".to_string(),
                }),
                amount: Some(common_v1::MonetaryValue { value: d.amount.to_string(), currency_code: d.currency }),
                payee_name: d.payee_name.unwrap_or_default(),
            }).collect(),
            exceptions: vec![],
            audit_data: None,
        }))
    }

    // Stubs -> Implemented
    async fn list_bank_statements(
        &self,
        request: Request<ListBankStatementsRequest>,
    ) -> Result<Response<ListBankStatementsResponse>, Status> {
        let req = request.into_inner();
        let pagination = req.pagination.unwrap_or_default();
        let limit = pagination.page_size.max(10).min(100) as i64;
        let offset = (pagination.page.max(1) - 1) as i64 * limit;
        let company_code = if req.company_code.is_empty() { None } else { Some(req.company_code.as_str()) };

        let statements = self.repo.list_statements(company_code, limit, offset).await
            .map_err(|e| Status::internal(e.to_string()))?;

        let total_items = self.repo.count_statements(company_code).await
            .map_err(|e| Status::internal(e.to_string()))?;
        let total_pages = if limit > 0 {
            ((total_items + limit - 1) / limit) as i32
        } else {
            1
        };

        let current_page = (offset / limit + 1) as i32;
        Ok(Response::new(ListBankStatementsResponse {
            statements: statements.into_iter().map(|s| BankStatementSummary {
                statement_id: s.statement_id.to_string(),
                statement_date: Some(prost_types::Timestamp {
                    seconds: s.created_at.timestamp(),
                    nanos: 0,
                }),
                company_code: s.company_code,
            }).collect(),
            pagination: Some(common_v1::PaginationResponse {
                current_page,
                page_size: limit as i32,
                total_items: total_items,
                total_pages,
            }),
        }))
    }

    async fn cancel_payment_run(
        &self,
        request: Request<CancelPaymentRunRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        let run_id = uuid::Uuid::parse_str(&req.run_id)
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;

        // Update status to CANCELLED
        self.repo.update_run_status(run_id, "CANCELLED").await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(common_v1::JobInfo {
            job_id: req.run_id,
            job_type: "PAYMENT_RUN".to_string(),
            status: common_v1::JobStatus::Cancelled as i32,
            progress_percentage: 0,
            messages: vec![common_v1::ApiMessage {
                r#type: "INFO".to_string(),
                code: "".to_string(),
                message: "Payment run cancelled".to_string(),
                target: "".to_string(),
            }],
            error_detail: "".to_string(),
            created_at: None,
            started_at: None,
            completed_at: None,
        }))
    }

    async fn list_payment_runs(
        &self,
        request: Request<ListPaymentRunsRequest>,
    ) -> Result<Response<ListPaymentRunsResponse>, Status> {
        let req = request.into_inner();
        let pagination = req.pagination.unwrap_or_default();
        let limit = pagination.page_size.max(10).min(100) as i64;
        let offset = (pagination.page.max(1) - 1) as i64 * limit;

        let runs = self.repo.list_payment_runs(None, limit, offset).await
            .map_err(|e| Status::internal(e.to_string()))?;

        let current_page = (offset / limit + 1) as i32;
        Ok(Response::new(ListPaymentRunsResponse {
            payment_runs: runs.into_iter().map(|r| PaymentRunSummary {
                run_id: r.run_id.to_string(),
                run_date: Some(prost_types::Timestamp {
                    seconds: r.created_at.timestamp(),
                    nanos: 0,
                }),
                status: match r.status.as_str() {
                    "COMPLETED" => common_v1::JobStatus::Completed as i32,
                    "RUNNING" => common_v1::JobStatus::Running as i32,
                    "CANCELLED" => common_v1::JobStatus::Cancelled as i32,
                    "FAILED" => common_v1::JobStatus::Failed as i32,
                    _ => common_v1::JobStatus::Pending as i32,
                },
            }).collect(),
            pagination: Some(common_v1::PaginationResponse {
                current_page,
                page_size: limit as i32,
                total_items: 0,
                total_pages: 1,
            }),
        }))
    }

    async fn get_job_status(
        &self,
        request: Request<GetJobStatusRequest>,
    ) -> Result<Response<common_v1::JobInfo>, Status> {
        let req = request.into_inner();
        
        // Try to find as payment run
        if let Ok(run_id) = uuid::Uuid::parse_str(&req.job_id) {
            if let Some(run) = self.repo.find_run_by_id(run_id).await
                .map_err(|e| Status::internal(e.to_string()))? {
                return Ok(Response::new(common_v1::JobInfo {
                    job_id: run.run_id.to_string(),
                    job_type: "PAYMENT_RUN".to_string(),
                    status: match run.status.as_str() {
                        "COMPLETED" => common_v1::JobStatus::Completed as i32,
                        "RUNNING" => common_v1::JobStatus::Running as i32,
                        "CANCELLED" => common_v1::JobStatus::Cancelled as i32,
                        "FAILED" => common_v1::JobStatus::Failed as i32,
                        _ => common_v1::JobStatus::Pending as i32,
                    },
                    progress_percentage: if run.status == "COMPLETED" { 100 } else { 0 },
                    messages: vec![],
                    error_detail: "".to_string(),
                    created_at: None,
                    started_at: None,
                    completed_at: None,
                }));
            }
            
            // Try as bank statement
            if let Some(stmt) = self.repo.find_statement_by_id(run_id).await
                .map_err(|e| Status::internal(e.to_string()))? {
                return Ok(Response::new(common_v1::JobInfo {
                    job_id: stmt.statement_id.to_string(),
                    job_type: "BANK_STATEMENT".to_string(),
                    status: match stmt.status.as_str() {
                        "PROCESSED" => common_v1::JobStatus::Completed as i32,
                        "PROCESSING" => common_v1::JobStatus::Running as i32,
                        _ => common_v1::JobStatus::Pending as i32,
                    },
                    progress_percentage: if stmt.status == "PROCESSED" { 100 } else { 0 },
                    messages: vec![],
                    error_detail: "".to_string(),
                    created_at: None,
                    started_at: None,
                    completed_at: None,
                }));
            }
        }
        
        Err(Status::not_found("Job not found"))
    }
}
