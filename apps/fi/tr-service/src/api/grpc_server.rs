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

    // Stubs
    async fn list_bank_statements(&self, _r: Request<ListBankStatementsRequest>) -> Result<Response<ListBankStatementsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn cancel_payment_run(&self, _r: Request<CancelPaymentRunRequest>) -> Result<Response<common_v1::JobInfo>, Status> { Err(Status::unimplemented("")) }
    async fn list_payment_runs(&self, _r: Request<ListPaymentRunsRequest>) -> Result<Response<ListPaymentRunsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn get_job_status(&self, _r: Request<GetJobStatusRequest>) -> Result<Response<common_v1::JobInfo>, Status> { Err(Status::unimplemented("")) }
}
