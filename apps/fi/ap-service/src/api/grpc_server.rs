//! AP gRPC Service Implementation

use tonic::{Request, Response, Status};
use std::sync::Arc;

use crate::application::commands::{PostSupplierCommand, ListOpenItemsQuery, PostInvoiceCommand};
use crate::application::handlers::{PostSupplierHandler, ListOpenItemsHandler, PostInvoiceHandler, GetInvoiceHandler, ApproveInvoiceHandler, RejectInvoiceHandler};

// Use the properly structured proto modules
use crate::api::proto::fi::ap::v1 as ap_v1;
use crate::api::proto::common::v1 as common_v1;

use ap_v1::accounts_receivable_payable_service_server::AccountsReceivablePayableService;
use ap_v1::*;
use common_v1::*;

use chrono::Datelike;
use std::str::FromStr;

/// gRPC Service Implementation
pub struct ApServiceImpl {
    post_supplier_handler: Arc<PostSupplierHandler>,
    list_open_items_handler: Arc<ListOpenItemsHandler>,
    post_invoice_handler: Arc<PostInvoiceHandler>,
    get_invoice_handler: Arc<GetInvoiceHandler>,
    approve_invoice_handler: Arc<ApproveInvoiceHandler>,
    reject_invoice_handler: Arc<RejectInvoiceHandler>,
}

impl ApServiceImpl {
    pub fn new(
        post_supplier_handler: Arc<PostSupplierHandler>,
        list_open_items_handler: Arc<ListOpenItemsHandler>,
        post_invoice_handler: Arc<PostInvoiceHandler>,
        get_invoice_handler: Arc<GetInvoiceHandler>,
        approve_invoice_handler: Arc<ApproveInvoiceHandler>,
        reject_invoice_handler: Arc<RejectInvoiceHandler>,
    ) -> Self {
        Self {
            post_supplier_handler,
            list_open_items_handler,
            post_invoice_handler,
            get_invoice_handler,
            approve_invoice_handler,
            reject_invoice_handler,
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
                    document_number: item.document_number,
                    fiscal_year: item.fiscal_year,
                    company_code: item.company_code,
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
        request: Request<GetAccountBalanceRequest>,
    ) -> Result<Response<GetAccountBalanceResponse>, Status> {
        let req = request.into_inner();

        // List all open items for the business partner
        let query = ListOpenItemsQuery {
            business_partner_id: req.business_partner_id.clone(),
            company_code: req.company_code.clone(),
            account_type: "SUPPLIER".to_string(),
            include_cleared: false,
            page_size: 1000,
            page_token: None,
        };

        let items = self.list_open_items_handler.handle(query).await?;

        // Calculate total balance
        let mut total_balance = rust_decimal::Decimal::ZERO;
        let currency = items.first().map(|i| i.currency.clone()).unwrap_or_else(|| "CNY".to_string());

        for item in &items {
            total_balance += item.open_amount;
        }

        Ok(Response::new(GetAccountBalanceResponse {
            balance: Some(to_proto_money(total_balance, &currency)),
        }))
    }


    async fn get_aging_analysis(
        &self,
        request: Request<GetAgingAnalysisRequest>,
    ) -> Result<Response<GetAgingAnalysisResponse>, Status> {
        let req = request.into_inner();

        // List all open items
        let query = ListOpenItemsQuery {
            business_partner_id: req.business_partner_id.clone(),
            company_code: req.company_code.clone(),
            account_type: "SUPPLIER".to_string(),
            include_cleared: false,
            page_size: 1000,
            page_token: None,
        };

        let items = self.list_open_items_handler.handle(query).await?;

        // Calculate aging buckets
        let today = chrono::Utc::now().naive_utc().date();
        let mut current = rust_decimal::Decimal::ZERO;
        let mut days_1_30 = rust_decimal::Decimal::ZERO;
        let mut days_31_60 = rust_decimal::Decimal::ZERO;
        let mut days_61_90 = rust_decimal::Decimal::ZERO;
        let mut days_over_90 = rust_decimal::Decimal::ZERO;

        for item in &items {
            let days_overdue = (today - item.due_date).num_days();

            if days_overdue <= 0 {
                current += item.open_amount;
            } else if days_overdue <= 30 {
                days_1_30 += item.open_amount;
            } else if days_overdue <= 60 {
                days_31_60 += item.open_amount;
            } else if days_overdue <= 90 {
                days_61_90 += item.open_amount;
            } else {
                days_over_90 += item.open_amount;
            }
        }

        let currency = items.first().map(|i| i.currency.clone()).unwrap_or_else(|| "CNY".to_string());
        let total = current + days_1_30 + days_31_60 + days_61_90 + days_over_90;

        Ok(Response::new(GetAgingAnalysisResponse {
            analysis: Some(ap_v1::AgingAnalysis {
                as_of_date: Some(prost_types::Timestamp {
                    seconds: today.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                    nanos: 0,
                }),
                total_open_amount: Some(to_proto_money(total, &currency)),
                overdue_buckets: vec![
                    ap_v1::AgingBucket {
                        days_from: 0,
                        days_to: 0,
                        amount: Some(to_proto_money(current, &currency)),
                    },
                    ap_v1::AgingBucket {
                        days_from: 1,
                        days_to: 30,
                        amount: Some(to_proto_money(days_1_30, &currency)),
                    },
                    ap_v1::AgingBucket {
                        days_from: 31,
                        days_to: 60,
                        amount: Some(to_proto_money(days_31_60, &currency)),
                    },
                    ap_v1::AgingBucket {
                        days_from: 61,
                        days_to: 90,
                        amount: Some(to_proto_money(days_61_90, &currency)),
                    },
                    ap_v1::AgingBucket {
                        days_from: 91,
                        days_to: 999,
                        amount: Some(to_proto_money(days_over_90, &currency)),
                    },
                ],
            }),
        }))
    }

    // ----------------------------------------------------------------
    // Invoices
    // ----------------------------------------------------------------

    async fn post_invoice(
        &self,
        request: Request<PostInvoiceRequest>,
    ) -> Result<Response<PostInvoiceResponse>, Status> {
        let req = request.into_inner();

        // Defaults since Proto doesn't have these fields
        let now = chrono::Utc::now().date_naive();
        // Assuming first item currency or default CNY
        let currency = req.items.first()
            .and_then(|i| i.amount.as_ref())
            .map(|a| a.currency_code.clone())
            .unwrap_or_else(|| "CNY".to_string());

        let cmd = crate::application::commands::PostInvoiceCommand {
            company_code: req.company_code.clone(),
            supplier_id: req.account_id, // Assuming account_id is supplier_id
            document_date: now,
            posting_date: now,
            currency,
            reference_document: None,
            header_text: None,
            items: req.items.into_iter().map(|item| {
                crate::application::commands::InvoiceItemCommand {
                    gl_account: item.gl_account,
                    debit_credit: item.debit_credit_indicator,
                    amount: rust_decimal::Decimal::from_str(&item.amount.unwrap_or_default().value).unwrap_or_default(),
                    cost_center: if item.cost_center.is_empty() { None } else { Some(item.cost_center) },
                    item_text: if item.item_text.is_empty() { None } else { Some(item.item_text) },
                    purchase_order: None, 
                    po_item_number: None,
                }
            }).collect(),
        };

        let invoice = self.post_invoice_handler.handle(cmd).await?;

        Ok(Response::new(PostInvoiceResponse {
            document: Some(common_v1::SystemDocumentReference {
                document_number: invoice.document_number,
                fiscal_year: invoice.fiscal_year,
                company_code: invoice.company_code,
                document_type: "KR".to_string(),
                document_category: "".to_string(),
            }),
        }))
    }

    async fn reverse_document(
        &self,
        _request: Request<ReverseDocumentRequest>,
    ) -> Result<Response<ReverseDocumentResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn verify_invoice(
        &self,
        request: Request<VerifyInvoiceRequest>,
    ) -> Result<Response<VerifyInvoiceResponse>, Status> {
        let req = request.into_inner();

        // Get document reference
        let doc_ref = req.document.ok_or_else(|| Status::invalid_argument("Missing document reference"))?;

        // For simplicity, we'll just return success
        // In real implementation, this would validate against purchase orders, goods receipts, etc.
        Ok(Response::new(VerifyInvoiceResponse {
            success: true,
        }))
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
    async fn post_sales_invoice(&self, _r: Request<PostSalesInvoiceRequest>) -> Result<Response<PostSalesInvoiceResponse>, Status> { Err(Status::unimplemented("Handled by AR Service")) }
}
