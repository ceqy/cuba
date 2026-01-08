use std::sync::Arc;
use tonic::{Request, Response, Status};

pub mod error;

use crate::proto::enterprise::finance::gl::*;
use crate::proto::enterprise::finance::gl::gl_journal_entry_service_server::GlJournalEntryService;
use sqlx::PgPool;

pub struct GlJournalEntryServiceImpl {
    #[allow(dead_code)]
    pool: Arc<PgPool>,
}

impl GlJournalEntryServiceImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl GlJournalEntryService for GlJournalEntryServiceImpl {
    async fn create_journal_entry(&self, _request: Request<CreateJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn batch_create_journal_entries(&self, _request: Request<BatchCreateJournalEntriesRequest>) -> Result<Response<BatchCreateJournalEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn validate_journal_entry(&self, _request: Request<ValidateJournalEntryRequest>) -> Result<Response<ValidationResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn simulate_journal_entry(&self, _request: Request<SimulateJournalEntryRequest>) -> Result<Response<SimulationResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_journal_entry(&self, _request: Request<GetJournalEntryRequest>) -> Result<Response<JournalEntryDetail>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn list_journal_entries(&self, _request: Request<ListJournalEntriesRequest>) -> Result<Response<ListJournalEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_journal_entries_by_account(&self, _request: Request<GetJournalEntriesByAccountRequest>) -> Result<Response<ListJournalEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_journal_entry_history(&self, _request: Request<GetJournalEntryHistoryRequest>) -> Result<Response<JournalEntryHistoryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn update_journal_entry(&self, _request: Request<UpdateJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn delete_journal_entry(&self, _request: Request<DeleteJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn post_journal_entry(&self, _request: Request<PostJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn cancel_journal_entry(&self, _request: Request<CancelJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reset_journal_entry(&self, _request: Request<ResetJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reverse_journal_entry(&self, _request: Request<ReverseJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn batch_reverse_journal_entries(&self, _request: Request<BatchReverseJournalEntriesRequest>) -> Result<Response<BatchReverseJournalEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn clear_open_items(&self, _request: Request<ClearOpenItemsRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reset_clearing(&self, _request: Request<ResetClearingRequest>) -> Result<Response<ResetClearingResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn list_open_items(&self, _request: Request<ListOpenItemsRequest>) -> Result<Response<ListOpenItemsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_journal_entry_tax_details(&self, _request: Request<GetJournalEntryTaxDetailsRequest>) -> Result<Response<JournalEntryTaxDetailsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn recalculate_tax(&self, _request: Request<RecalculateTaxRequest>) -> Result<Response<RecalculateTaxResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_payment_information(&self, _request: Request<GetPaymentInformationRequest>) -> Result<Response<PaymentInformationResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn update_payment_status(&self, _request: Request<UpdatePaymentStatusRequest>) -> Result<Response<UpdatePaymentStatusResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn upload_attachment(&self, _request: Request<UploadAttachmentRequest>) -> Result<Response<UploadAttachmentResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn download_attachment(&self, _request: Request<DownloadAttachmentRequest>) -> Result<Response<DownloadAttachmentResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn list_attachments(&self, _request: Request<ListAttachmentsRequest>) -> Result<Response<ListAttachmentsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn delete_attachment(&self, _request: Request<DeleteAttachmentRequest>) -> Result<Response<DeleteAttachmentResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn submit_for_approval(&self, _request: Request<SubmitForApprovalRequest>) -> Result<Response<SubmitForApprovalResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn approve_journal_entry(&self, _request: Request<ApproveJournalEntryRequest>) -> Result<Response<ApproveJournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reject_journal_entry(&self, _request: Request<RejectJournalEntryRequest>) -> Result<Response<RejectJournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_approval_history(&self, _request: Request<GetApprovalHistoryRequest>) -> Result<Response<ApprovalHistoryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn export_journal_entries(&self, _request: Request<ExportJournalEntriesRequest>) -> Result<Response<ExportJournalEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_journal_entry_statistics(&self, _request: Request<GetJournalEntryStatisticsRequest>) -> Result<Response<JournalEntryStatisticsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn generate_print_preview(&self, _request: Request<GeneratePrintPreviewRequest>) -> Result<Response<GeneratePrintPreviewResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn park_journal_entry(&self, _request: Request<ParkJournalEntryRequest>) -> Result<Response<ParkJournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn list_parked_journal_entries(&self, _request: Request<ListParkedJournalEntriesRequest>) -> Result<Response<ListParkedJournalEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn post_parked_journal_entry(&self, _request: Request<PostParkedJournalEntryRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn create_recurring_entry(&self, _request: Request<CreateRecurringEntryRequest>) -> Result<Response<RecurringEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn execute_recurring_entry(&self, _request: Request<ExecuteRecurringEntryRequest>) -> Result<Response<ExecuteRecurringEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn list_recurring_entries(&self, _request: Request<ListRecurringEntriesRequest>) -> Result<Response<ListRecurringEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn save_as_template(&self, _request: Request<SaveAsTemplateRequest>) -> Result<Response<SaveAsTemplateResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn create_from_template(&self, _request: Request<CreateFromTemplateRequest>) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn list_templates(&self, _request: Request<ListTemplatesRequest>) -> Result<Response<ListTemplatesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_document_chain(&self, _request: Request<GetDocumentChainRequest>) -> Result<Response<DocumentChainResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reconcile_documents(&self, _request: Request<ReconcileDocumentsRequest>) -> Result<Response<ReconcileDocumentsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn execute_period_end_close(&self, _request: Request<ExecutePeriodEndCloseRequest>) -> Result<Response<PeriodEndCloseResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn revaluate_foreign_currency(&self, _request: Request<RevaluateForeignCurrencyRequest>) -> Result<Response<RevaluationResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reclassify_accounts(&self, _request: Request<ReclassifyAccountsRequest>) -> Result<Response<ReclassificationResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn carry_forward_balances(&self, _request: Request<CarryForwardBalancesRequest>) -> Result<Response<CarryForwardBalancesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn create_batch_input_session(&self, _request: Request<CreateBatchInputSessionRequest>) -> Result<Response<BatchInputSessionResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn process_batch_input_session(&self, _request: Request<ProcessBatchInputSessionRequest>) -> Result<Response<ProcessBatchInputSessionResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_batch_input_session_status(&self, _request: Request<GetBatchInputSessionStatusRequest>) -> Result<Response<BatchInputSessionStatusResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_document_split_details(&self, _request: Request<GetDocumentSplitDetailsRequest>) -> Result<Response<DocumentSplitDetailsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn configure_document_splitting(&self, _request: Request<ConfigureDocumentSplittingRequest>) -> Result<Response<ConfigureDocumentSplittingResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_parallel_ledger_data(&self, _request: Request<GetParallelLedgerDataRequest>) -> Result<Response<ParallelLedgerDataResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn adjust_parallel_ledger(&self, _request: Request<AdjustParallelLedgerRequest>) -> Result<Response<AdjustParallelLedgerResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_account_balances(&self, _request: Request<GetAccountBalancesRequest>) -> Result<Response<AccountBalancesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_account_line_items(&self, _request: Request<GetAccountLineItemsRequest>) -> Result<Response<AccountLineItemsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reconcile_gl_and_subledger(&self, _request: Request<ReconcileGlAndSubledgerRequest>) -> Result<Response<ReconciliationResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn generate_journal_analysis(&self, _request: Request<GenerateJournalAnalysisRequest>) -> Result<Response<JournalAnalysisResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }
}
