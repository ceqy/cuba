//! AP gRPC Service Implementation

use tonic::{Request, Response, Status};
use std::sync::Arc;

use crate::application::commands::{PostSupplierCommand, ListOpenItemsQuery};
use crate::application::handlers::{PostSupplierHandler, ListOpenItemsHandler};

// Use the properly structured proto modules
use crate::api::proto::fi::ap::v1 as ap_v1;
use crate::api::proto::common::v1 as common_v1;

use ap_v1::accounts_receivable_payable_service_server::AccountsReceivablePayableService;
use ap_v1::*;
use common_v1::*;

/// gRPC Service Implementation
pub struct ApServiceImpl {
    post_supplier_handler: Arc<PostSupplierHandler>,
    list_open_items_handler: Arc<ListOpenItemsHandler>,
}

impl ApServiceImpl {
    pub fn new(
        post_supplier_handler: Arc<PostSupplierHandler>,
        list_open_items_handler: Arc<ListOpenItemsHandler>,
    ) -> Self {
        Self {
            post_supplier_handler,
            list_open_items_handler,
        }
    }
}

// Monetary Helper specific to generated types
fn to_proto_money(amount: rust_decimal::Decimal, currency: &str) -> common_v1::MonetaryValue {
    common_v1::MonetaryValue {
        value: amount.to_string(),
        currency_code: currency.to_string(),
    }
}

#[tonic::async_trait]
impl AccountsReceivablePayableService for ApServiceImpl {
    // ----------------------------------------------------------------
    // Master Data
    // ----------------------------------------------------------------

    async fn post_supplier(
        &self,
        request: Request<SupplierDetails>,
    ) -> Result<Response<SupplierDetails>, Status> {
        let req = request.into_inner();
        
        // Handle optional embedded Address message
        let (street, city, postal_code, country) = if let Some(addr) = &req.address {
            (
                Some(addr.street.clone()),
                Some(addr.city.clone()),
                Some(addr.postal_code.clone()),
                Some(addr.country.clone()),
            )
        } else {
            (None, None, None, None)
        };
        
        let cmd = PostSupplierCommand {
            supplier_id: req.supplier_id,
            business_partner_id: Some(req.business_partner_id),
            name: req.name,
            account_group: req.account_group,
            street,
            city,
            postal_code,
            country,
            telephone: Some(req.telephone),
            email: Some(req.email),
            company_code: req.company_code,
            reconciliation_account: req.reconciliation_account,
            payment_terms: Some(req.payment_terms),
            check_double_invoice: req.check_double_invoice,
            purchasing_organization: Some(req.purchasing_organization),
            order_currency: Some(req.order_currency),
        };

        let supplier = self.post_supplier_handler.handle(cmd).await?;

        Ok(Response::new(SupplierDetails {
            supplier_id: supplier.supplier_id,
            business_partner_id: supplier.business_partner_id.unwrap_or_default(),
            name: supplier.name,
            account_group: supplier.account_group,
            company_code: supplier.company_code,
            reconciliation_account: supplier.reconciliation_account,
            ..Default::default() 
        }))
    }

    async fn post_customer(
        &self,
        _request: Request<CustomerDetails>,
    ) -> Result<Response<CustomerDetails>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }


    async fn get_partner_details(
        &self,
        _request: Request<GetPartnerDetailsRequest>,
    ) -> Result<Response<GetPartnerDetailsResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn batch_get_partner_details(
        &self,
        _request: Request<BatchGetPartnerDetailsRequest>,
    ) -> Result<Response<BatchGetPartnerDetailsResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    // ----------------------------------------------------------------
    // Open Items & Analysis
    // ----------------------------------------------------------------

    async fn list_open_items(
        &self,
        request: Request<ListOpenItemsRequest>,
    ) -> Result<Response<ListOpenItemsResponse>, Status> {
        let req = request.into_inner();
        
        let query = ListOpenItemsQuery {
            business_partner_id: req.business_partner_id,
            company_code: req.company_code,
            account_type: req.account_type,
            include_cleared: req.filter.map(|f| f.include_cleared).unwrap_or(false),
            page_size: req.pagination.as_ref().map(|p| p.page_size).unwrap_or(20),
            page_token: None,
        };

        let items = self.list_open_items_handler.handle(query).await?;

        let proto_items = items.into_iter().map(|item| {
            OpenItem {
                document_reference: Some(common_v1::SystemDocumentReference {
                    // system_id: "SAP_S4".to_string(), // Removed
                    document_number: item.document_number,
                    fiscal_year: item.fiscal_year, // Correct type: i32
                    company_code: item.company_code,
                    // item_number: item.line_item_number.to_string(), // Removed
                    document_type: "KR".to_string(), // Default to Vendor Invoice
                    document_category: "".to_string(),
                }),
                line_item_number: item.line_item_number,
                posting_date: Some(prost_types::Timestamp {
                    seconds: item.posting_date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                    nanos: 0,
                }),
                due_date: Some(prost_types::Timestamp {
                    seconds: item.due_date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                    nanos: 0,
                }),
                amount: Some(to_proto_money(item.original_amount, &item.currency)),
                open_amount: Some(to_proto_money(item.open_amount, &item.currency)),
                gl_account: "".to_string(), 
                payment_block: item.payment_block.unwrap_or_default(),
                reference_document: item.reference_document.unwrap_or_default(),
                item_text: item.item_text.unwrap_or_default(),
                installments: vec![],
                ledger: None,
            }
        }).collect();

        Ok(Response::new(ListOpenItemsResponse {
            items: proto_items,
            pagination: None,
        }))
    }

    async fn get_account_balance(
        &self,
        _request: Request<GetAccountBalanceRequest>,
    ) -> Result<Response<GetAccountBalanceResponse>, Status> {
       Err(Status::unimplemented("Not implemented"))
    }


    async fn get_aging_analysis(
        &self,
        _request: Request<GetAgingAnalysisRequest>,
    ) -> Result<Response<GetAgingAnalysisResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    // ----------------------------------------------------------------
    // Invoices
    // ----------------------------------------------------------------

    async fn post_invoice(
        &self,
        _request: Request<PostInvoiceRequest>,
    ) -> Result<Response<PostInvoiceResponse>, Status> {
        // TODO: Implement PostInvoiceHandler
        Err(Status::unimplemented("Not implemented"))
    }

    async fn reverse_document(
        &self,
        _request: Request<ReverseDocumentRequest>,
    ) -> Result<Response<ReverseDocumentResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn verify_invoice(
        &self,
        _request: Request<VerifyInvoiceRequest>,
    ) -> Result<Response<VerifyInvoiceResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }


    // ----------------------------------------------------------------
    // Payments & Clearing (Stubs)
    // ----------------------------------------------------------------

    async fn generate_statement(&self, _r: Request<GenerateStatementRequest>) -> Result<Response<GenerateStatementResponse>, Status> { Err(Status::unimplemented("")) }
    async fn get_dunning_history(&self, _r: Request<GetDunningHistoryRequest>) -> Result<Response<GetDunningHistoryResponse>, Status> { Err(Status::unimplemented("")) }
    async fn trigger_dunning(&self, _r: Request<TriggerDunningRequest>) -> Result<Response<TriggerDunningResponse>, Status> { Err(Status::unimplemented("")) }
    async fn get_clearing_proposal(&self, _r: Request<GetClearingProposalRequest>) -> Result<Response<GetClearingProposalResponse>, Status> { Err(Status::unimplemented("")) }
    async fn execute_clearing_proposal(&self, _r: Request<ExecuteClearingProposalRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn clear_open_items(&self, _r: Request<ClearOpenItemsRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn partial_clear_items(&self, _r: Request<PartialClearItemsRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn net_clearing(&self, _r: Request<NetClearingRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn check_credit_limit(&self, _r: Request<CheckCreditLimitRequest>) -> Result<Response<CheckCreditLimitResponse>, Status> { Err(Status::unimplemented("")) }
    async fn update_credit_exposure(&self, _r: Request<UpdateCreditExposureRequest>) -> Result<Response<UpdateCreditExposureResponse>, Status> { Err(Status::unimplemented("")) }
    async fn generate_payment_proposal(&self, _r: Request<GeneratePaymentProposalRequest>) -> Result<Response<GeneratePaymentProposalResponse>, Status> { Err(Status::unimplemented("")) }
    async fn execute_payment_proposal(&self, _r: Request<ExecutePaymentProposalRequest>) -> Result<Response<PaymentExecutionResponse>, Status> { Err(Status::unimplemented("")) }
    async fn request_down_payment(&self, _r: Request<DownPaymentRequest>) -> Result<Response<DownPaymentResponse>, Status> { Err(Status::unimplemented("")) }
    async fn clear_down_payment(&self, _r: Request<DownPaymentClearingRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn list_attachments(&self, _r: Request<ListAttachmentsRequest>) -> Result<Response<ListAttachmentsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn upload_attachment(&self, _r: Request<UploadAttachmentRequest>) -> Result<Response<OperationResponse>, Status> { Err(Status::unimplemented("")) }
    async fn import_bank_statement(&self, _r: Request<ImportBankStatementRequest>) -> Result<Response<ImportBankStatementResponse>, Status> { Err(Status::unimplemented("")) }
    async fn process_lockbox(&self, _r: Request<ProcessLockboxRequest>) -> Result<Response<ProcessLockboxResponse>, Status> { Err(Status::unimplemented("")) }
    async fn apply_cash(&self, _r: Request<ApplyCashRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn get_tolerance_groups(&self, _r: Request<GetToleranceGroupsRequest>) -> Result<Response<GetToleranceGroupsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn perform_compliance_check(&self, _r: Request<PerformComplianceCheckRequest>) -> Result<Response<PerformComplianceCheckResponse>, Status> { Err(Status::unimplemented("")) }
    async fn export_report(&self, _r: Request<ExportReportRequest>) -> Result<Response<ExportReportResponse>, Status> { Err(Status::unimplemented("")) }
    async fn subscribe_to_events(&self, _r: Request<SubscribeToEventsRequest>) -> Result<Response<SubscribeToEventsResponse>, Status> { Err(Status::unimplemented("")) }
    async fn list_event_types(&self, _r: Request<ListEventTypesRequest>) -> Result<Response<ListEventTypesResponse>, Status> { Err(Status::unimplemented("")) }
}
