use crate::application::JournalEntryService;
use crate::infrastructure::PgJournalEntryRepository;
use crate::infrastructure::mapper;
use crate::proto::finance::gl::*;
use crate::proto::finance::gl::gl_journal_entry_service_server::GlJournalEntryService;
use sqlx::PgPool;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub mod error;

pub struct GlJournalEntryServiceImpl {
    service: Arc<JournalEntryService<PgJournalEntryRepository>>,
}

impl GlJournalEntryServiceImpl {
    pub fn new(service: Arc<JournalEntryService<PgJournalEntryRepository>>) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl GlJournalEntryService for GlJournalEntryServiceImpl {
    async fn create_journal_entry(
        &self,
        request: Request<CreateJournalEntryRequest>,
    ) -> Result<Response<JournalEntryResponse>, Status> {
        let req = request.into_inner();
        // TODO: Get user ID from metadata/context
        let user_id = Uuid::nil();

        let cmd = mapper::map_create_request(req, user_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let entry = self.service.create_journal_entry(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(mapper::map_to_response(&entry)))
    }

    async fn get_journal_entry(
        &self,
        request: Request<GetJournalEntryRequest>,
    ) -> Result<Response<JournalEntryDetail>, Status> {
        let req = request.into_inner();
        let id_str = req.journal_entry_id;
        
        let id = if id_str.is_empty() {
            if let Some(doc_ref) = req.document_reference {
                // Here we usually should allow lookup by company/year/number
                // but for now we only support lookup by UUID (document_id is missing in doc_ref proto but number might be a UUID string)
                Uuid::parse_str(&doc_ref.document_number)
                    .map_err(|_| Status::invalid_argument("Invalid document number format, UUID expected"))?
            } else {
                return Err(Status::invalid_argument("Either journal_entry_id or document_reference is required"));
            }
        } else {
            Uuid::parse_str(&id_str)
                .map_err(|_| Status::invalid_argument("Invalid journal entry ID"))?
        };

        let entry = self.service.get_journal_entry(id).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Journal entry not found"))?;

        Ok(Response::new(mapper::map_to_detail(&entry)))
    }

    async fn list_journal_entries(
        &self,
        request: Request<ListJournalEntriesRequest>,
    ) -> Result<Response<ListJournalEntriesResponse>, Status> {
        let req = request.into_inner();
        
        let filter = crate::domain::repository::JournalEntryFilter {
            company_code: if req.company_code.is_empty() { None } else { Some(req.company_code) },
            fiscal_year: if req.fiscal_year == 0 { None } else { Some(req.fiscal_year) },
            ..Default::default()
        };

        let pagination = crate::domain::repository::Pagination {
            page: req.pagination.as_ref().map(|p| p.page as u32).unwrap_or(1),
            page_size: req.pagination.as_ref().map(|p| p.page_size as u32).unwrap_or(20),
        };

        let result = self.service.list_journal_entries(filter, pagination).await
            .map_err(|e| Status::internal(e.to_string()))?;

        let total_count = result.total_count;
        let total_pages = result.total_pages();
        let page = result.page;
        let page_size = result.page_size;
        let entries = mapper::map_items_to_summary(result.items);

        Ok(Response::new(ListJournalEntriesResponse {
            entries,
            pagination: Some(crate::proto::common::PaginationResponse {
                current_page: page as i32,
                page_size: page_size as i32,
                total_items: total_count as i64,
                total_pages: total_pages as i32,
            }),
        }))
    }

    async fn update_journal_entry(
        &self,
        request: Request<UpdateJournalEntryRequest>,
    ) -> Result<Response<JournalEntryResponse>, Status> {
        let req = request.into_inner();
        let cmd = mapper::map_update_request(req)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let entry = self.service.update_journal_entry(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(mapper::map_to_response(&entry)))
    }

    async fn delete_journal_entry(
        &self,
        request: Request<DeleteJournalEntryRequest>,
    ) -> Result<Response<JournalEntryResponse>, Status> {
        let req = request.into_inner();
        let id_str = req.journal_entry_id;
        let id = Uuid::parse_str(&id_str)
            .map_err(|_| Status::invalid_argument("Invalid journal entry ID"))?;

        let entry = self.service.get_journal_entry(id).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Journal entry not found"))?;

        if !entry.status().is_editable() {
            return Err(Status::failed_precondition("Cannot delete non-editable entry"));
        }

        self.service.delete_journal_entry(id).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(mapper::map_to_response(&entry)))
    }

    async fn post_journal_entry(
        &self,
        request: Request<PostJournalEntryRequest>,
    ) -> Result<Response<JournalEntryResponse>, Status> {
        let req = request.into_inner();
        let id_str = req.journal_entry_id;
        let id = Uuid::parse_str(&id_str)
            .map_err(|_| Status::invalid_argument("Invalid journal entry ID"))?;

        // TODO: Get user ID from metadata/context
        let user_id = Uuid::nil();

        let entry = self.service.post_journal_entry(id, user_id).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(mapper::map_to_response(&entry)))
    }

    async fn reverse_journal_entry(
        &self,
        request: Request<ReverseJournalEntryRequest>,
    ) -> Result<Response<JournalEntryResponse>, Status> {
        let req = request.into_inner();
        let id_str = req.journal_entry_id;
        let id = Uuid::parse_str(&id_str)
            .map_err(|_| Status::invalid_argument("Invalid journal entry ID"))?;
            
        let reason = req.reversal_reason;
        let posting_date = req.posting_date.map(|ts| {
            chrono::NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as u32)
                .map(|dt| dt.date())
        }).flatten();

        // TODO: Get user ID from metadata/context
        let user_id = Uuid::nil();

        let (_original, reversal) = self.service.reverse_journal_entry(
            id,
            &reason,
            posting_date,
            user_id
        ).await.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(mapper::map_to_response(&reversal)))
    }

    async fn park_journal_entry(
        &self,
        request: Request<ParkJournalEntryRequest>,
    ) -> Result<Response<ParkJournalEntryResponse>, Status> {
        let req = request.into_inner();
        let command = mapper::map_park_request(req)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let entry = self.service.park_journal_entry(command).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(mapper::map_to_park_response(&entry)))
    }

    async fn post_parked_journal_entry(
        &self,
        request: Request<PostParkedJournalEntryRequest>,
    ) -> Result<Response<JournalEntryResponse>, Status> {
        let req = request.into_inner();
        let id_str = req.parked_journal_entry_id;
        let id = Uuid::parse_str(&id_str)
            .map_err(|_| Status::invalid_argument("Invalid journal entry ID"))?;

        // TODO: Get user ID from metadata/context
        let user_id = Uuid::nil();

        let entry = self.service.post_journal_entry(id, user_id).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(mapper::map_to_response(&entry)))
    }

    async fn get_journal_entry_history(
        &self,
        _request: Request<GetJournalEntryHistoryRequest>,
    ) -> Result<Response<JournalEntryHistoryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn list_parked_journal_entries(
        &self,
        _request: Request<ListParkedJournalEntriesRequest>,
    ) -> Result<Response<ListParkedJournalEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn validate_journal_entry(
        &self,
        _request: Request<ValidateJournalEntryRequest>,
    ) -> Result<Response<ValidationResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn simulate_journal_entry(
        &self,
        _request: Request<SimulateJournalEntryRequest>,
    ) -> Result<Response<SimulationResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_account_balances(
        &self,
        _request: Request<GetAccountBalancesRequest>,
    ) -> Result<Response<AccountBalancesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_account_line_items(
        &self,
        _request: Request<GetAccountLineItemsRequest>,
    ) -> Result<Response<AccountLineItemsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reconcile_gl_and_subledger(
        &self,
        _request: Request<ReconcileGlAndSubledgerRequest>,
    ) -> Result<Response<ReconciliationResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn generate_journal_analysis(
        &self,
        _request: Request<GenerateJournalAnalysisRequest>,
    ) -> Result<Response<JournalAnalysisResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reset_journal_entry(
        &self,
        _request: Request<ResetJournalEntryRequest>,
    ) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn submit_for_approval(
        &self,
        _request: Request<SubmitForApprovalRequest>,
    ) -> Result<Response<SubmitForApprovalResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reject_journal_entry(
        &self,
        _request: Request<RejectJournalEntryRequest>,
    ) -> Result<Response<RejectJournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_approval_history(
        &self,
        _request: Request<GetApprovalHistoryRequest>,
    ) -> Result<Response<ApprovalHistoryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn upload_attachment(
        &self,
        _request: Request<UploadAttachmentRequest>,
    ) -> Result<Response<UploadAttachmentResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn download_attachment(
        &self,
        _request: Request<DownloadAttachmentRequest>,
    ) -> Result<Response<DownloadAttachmentResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn list_attachments(
        &self,
        _request: Request<ListAttachmentsRequest>,
    ) -> Result<Response<ListAttachmentsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn delete_attachment(
        &self,
        _request: Request<DeleteAttachmentRequest>,
    ) -> Result<Response<DeleteAttachmentResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn create_from_template(
        &self,
        _request: Request<CreateFromTemplateRequest>,
    ) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn save_as_template(
        &self,
        _request: Request<SaveAsTemplateRequest>,
    ) -> Result<Response<SaveAsTemplateResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn list_templates(
        &self,
        _request: Request<ListTemplatesRequest>,
    ) -> Result<Response<ListTemplatesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn create_recurring_entry(
        &self,
        _request: Request<CreateRecurringEntryRequest>,
    ) -> Result<Response<RecurringEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn list_recurring_entries(
        &self,
        _request: Request<ListRecurringEntriesRequest>,
    ) -> Result<Response<ListRecurringEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn execute_recurring_entry(
        &self,
        _request: Request<ExecuteRecurringEntryRequest>,
    ) -> Result<Response<ExecuteRecurringEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn revaluate_foreign_currency(
        &self,
        _request: Request<RevaluateForeignCurrencyRequest>,
    ) -> Result<Response<RevaluationResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_parallel_ledger_data(
        &self,
        _request: Request<GetParallelLedgerDataRequest>,
    ) -> Result<Response<ParallelLedgerDataResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn carry_forward_balances(
        &self,
        _request: Request<CarryForwardBalancesRequest>,
    ) -> Result<Response<CarryForwardBalancesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn execute_period_end_close(
        &self,
        _request: Request<ExecutePeriodEndCloseRequest>,
    ) -> Result<Response<PeriodEndCloseResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_journal_entry_tax_details(
        &self,
        _request: Request<GetJournalEntryTaxDetailsRequest>,
    ) -> Result<Response<JournalEntryTaxDetailsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn recalculate_tax(
        &self,
        request: Request<RecalculateTaxRequest>,
    ) -> Result<Response<RecalculateTaxResponse>, Status> {
        let req = request.into_inner();
        let doc_ref = req.document_reference.ok_or_else(|| Status::invalid_argument("Missing document reference"))?;
        let id = Uuid::parse_str(&doc_ref.document_number)
            .map_err(|_| Status::invalid_argument("Invalid document number as UUID"))?;

        // 1. 获取凭证
        let mut entry = self.service.get_journal_entry(id).await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Journal entry not found"))?;

        // 2. 重新计算税务
        let tax_service = crate::application::MockTaxService;
        entry.calculate_taxes(&tax_service)
            .map_err(|e| Status::internal(format!("Tax calculation failed: {:?}", e)))?;

        // 3. 保存凭证 (需要通过 service 保存)
        // Note: 这可能需要增加一个 update 方法

        Ok(Response::new(RecalculateTaxResponse {
            success: true,
            recalculated_tax_items: entry.tax_items().iter().map(|t| {
                TaxLineItem {
                    line_item_number: t.line_number,
                    tax_code: t.tax_code.clone(),
                    tax_rate: t.tax_rate.to_string(),
                    tax_amount_doc: t.tax_amount.to_string(),
                    ..Default::default()
                }
            }).collect(),
            messages: vec![],
        }))
    }

    async fn list_open_items(
        &self,
        _request: Request<ListOpenItemsRequest>,
    ) -> Result<Response<ListOpenItemsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn clear_open_items(
        &self,
        request: Request<ClearOpenItemsRequest>,
    ) -> Result<Response<ClearOpenItemsResponse>, Status> {
        let req = request.into_inner();
        
        // 从 items_to_clear 提取行项目信息
        let line_ids: Vec<Uuid> = req.items_to_clear.iter()
            .filter_map(|item| {
                // 使用 document_number 作为 UUID (假设存储的是 UUID)
                Uuid::parse_str(&item.document_number).ok()
            })
            .collect();

        if line_ids.is_empty() {
            return Err(Status::invalid_argument("No valid items to clear"));
        }

        let clearing_date = chrono::Utc::now().naive_utc().date();
        
        let command = crate::application::ClearOpenItemsCommand {
            company_code: req.company_code.clone(),
            fiscal_year: req.items_to_clear.first().map(|i| i.fiscal_year).unwrap_or(2024),
            line_ids,
            clearing_date,
            currency: "CNY".to_string(), // 从上下文获取
            created_by: Uuid::nil(), // TODO: Get from context
        };

        let clearing_doc = self.service.clear_open_items(command).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ClearOpenItemsResponse {
            success: true,
            clearing_document_reference: Some(crate::proto::common::SystemDocumentReference {
                document_number: clearing_doc.clearing_number.clone(),
                fiscal_year: clearing_doc.fiscal_year,
                company_code: clearing_doc.company_code.clone(),
                document_type: "CLEAR".to_string(),
                document_category: "C".to_string(),
            }),
            messages: vec![],
            clearing_info: None,
        }))
    }

    async fn reset_clearing(
        &self,
        _request: Request<ResetClearingRequest>,
    ) -> Result<Response<ResetClearingResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_journal_entry_statistics(
        &self,
        _request: Request<GetJournalEntryStatisticsRequest>,
    ) -> Result<Response<JournalEntryStatisticsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn export_journal_entries(
        &self,
        _request: Request<ExportJournalEntriesRequest>,
    ) -> Result<Response<ExportJournalEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn generate_print_preview(
        &self,
        _request: Request<GeneratePrintPreviewRequest>,
    ) -> Result<Response<GeneratePrintPreviewResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_payment_information(
        &self,
        _request: Request<GetPaymentInformationRequest>,
    ) -> Result<Response<PaymentInformationResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn update_payment_status(
        &self,
        _request: Request<UpdatePaymentStatusRequest>,
    ) -> Result<Response<UpdatePaymentStatusResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_document_split_details(
        &self,
        _request: Request<GetDocumentSplitDetailsRequest>,
    ) -> Result<Response<DocumentSplitDetailsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn configure_document_splitting(
        &self,
        _request: Request<ConfigureDocumentSplittingRequest>,
    ) -> Result<Response<ConfigureDocumentSplittingResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn create_batch_input_session(
        &self,
        _request: Request<CreateBatchInputSessionRequest>,
    ) -> Result<Response<BatchInputSessionResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn process_batch_input_session(
        &self,
        _request: Request<ProcessBatchInputSessionRequest>,
    ) -> Result<Response<ProcessBatchInputSessionResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_batch_input_session_status(
        &self,
        _request: Request<GetBatchInputSessionStatusRequest>,
    ) -> Result<Response<BatchInputSessionStatusResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_document_chain(
        &self,
        _request: Request<GetDocumentChainRequest>,
    ) -> Result<Response<DocumentChainResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reclassify_accounts(
        &self,
        _request: Request<ReclassifyAccountsRequest>,
    ) -> Result<Response<ReclassificationResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn get_journal_entries_by_account(
        &self,
        _request: Request<GetJournalEntriesByAccountRequest>,
    ) -> Result<Response<ListJournalEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn reconcile_documents(
        &self,
        _request: Request<ReconcileDocumentsRequest>,
    ) -> Result<Response<ReconcileDocumentsResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn batch_create_journal_entries(
        &self,
        _request: Request<BatchCreateJournalEntriesRequest>,
    ) -> Result<Response<BatchCreateJournalEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn cancel_journal_entry(
        &self,
        _request: Request<CancelJournalEntryRequest>,
    ) -> Result<Response<JournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn batch_reverse_journal_entries(
        &self,
        _request: Request<BatchReverseJournalEntriesRequest>,
    ) -> Result<Response<BatchReverseJournalEntriesResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn approve_journal_entry(
        &self,
        _request: Request<ApproveJournalEntryRequest>,
    ) -> Result<Response<ApproveJournalEntryResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }

    async fn adjust_parallel_ledger(
        &self,
        _request: Request<AdjustParallelLedgerRequest>,
    ) -> Result<Response<AdjustParallelLedgerResponse>, Status> {
        Err(Status::unimplemented("not implemented"))
    }
}
