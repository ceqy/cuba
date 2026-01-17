use tonic::{Request, Response, Status};
use std::sync::Arc;
use std::str::FromStr;

use crate::application::commands::{PostCustomerCommand, ListOpenItemsQuery};
use crate::application::handlers::{PostCustomerHandler, ListOpenItemsHandler, PostSalesInvoiceHandler, ClearOpenItemsHandler, PartialClearHandler};

use crate::api::proto::fi::ap::v1 as ap_v1;
use crate::api::proto::common::v1 as common_v1;

use ap_v1::accounts_receivable_payable_service_server::AccountsReceivablePayableService;
use ap_v1::*;
use common_v1::*;

pub struct ArServiceImpl {
    post_customer_handler: Arc<PostCustomerHandler>,
    list_open_items_handler: Arc<ListOpenItemsHandler>,
    post_sales_invoice_handler: Arc<PostSalesInvoiceHandler>,
    clear_open_items_handler: Arc<ClearOpenItemsHandler>,
    partial_clear_handler: Arc<PartialClearHandler>,
}

impl ArServiceImpl {
    pub fn new(
        post_customer_handler: Arc<PostCustomerHandler>,
        list_open_items_handler: Arc<ListOpenItemsHandler>,
        post_sales_invoice_handler: Arc<PostSalesInvoiceHandler>,
        clear_open_items_handler: Arc<ClearOpenItemsHandler>,
        partial_clear_handler: Arc<PartialClearHandler>,
    ) -> Self {
        Self {
            post_customer_handler,
            list_open_items_handler,
            post_sales_invoice_handler,
            clear_open_items_handler,
            partial_clear_handler,
        }
    }
}

// Monetary Helper
fn to_proto_money(amount: rust_decimal::Decimal, currency: &str) -> common_v1::MonetaryValue {
    common_v1::MonetaryValue {
        value: amount.to_string(),
        currency_code: currency.to_string(),
    }
}

#[tonic::async_trait]
impl AccountsReceivablePayableService for ArServiceImpl {
    // ----------------------------------------------------------------
    // Customer Master Data
    // ----------------------------------------------------------------

    async fn post_customer(
        &self,
        request: Request<CustomerDetails>,
    ) -> Result<Response<CustomerDetails>, Status> {
        let req = request.into_inner();
        
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
        
        let cmd = PostCustomerCommand {
            customer_id: req.customer_id,
            business_partner_id: Some(req.business_partner_id),
            name: req.name,
            account_group: req.account_group,
            street,
            city,
            postal_code,
            country,
            company_code: req.company_code,
            reconciliation_account: req.reconciliation_account,
            payment_terms: Some(req.payment_terms),
            sales_organization: None, // Not in proto
            order_currency: None, // Not in proto
        };

        let customer = self.post_customer_handler.handle(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CustomerDetails {
            customer_id: customer.customer_id,
            business_partner_id: customer.business_partner_id.unwrap_or_default(),
            name: customer.name,
            account_group: customer.account_group,
            company_code: customer.company_code,
            reconciliation_account: customer.reconciliation_account,
            ..Default::default() 
        }))
    }

    async fn post_supplier(
        &self,
        _request: Request<SupplierDetails>,
    ) -> Result<Response<SupplierDetails>, Status> {
        Err(Status::unimplemented("Handled by AP Service"))
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
    // Open Items
    // ----------------------------------------------------------------

    async fn list_open_items(
        &self,
        request: Request<ListOpenItemsRequest>,
    ) -> Result<Response<ListOpenItemsResponse>, Status> {
        let req = request.into_inner();

        // AR Service only handles Customer open items. SAP "D" = Customer (Debitor)
        if req.account_type != "D" {
             return Err(Status::invalid_argument("AR Service requires Account Type 'D' (Customer)"));
        }

        if req.business_partner_id.is_empty() {
             return Err(Status::invalid_argument("Business Partner ID required"));
        }
        
        let query = ListOpenItemsQuery {
            // In MVP we assume BP ID = Customer ID for simplicity, or we'd query a mapping
            customer_id: req.business_partner_id,
            company_code: req.company_code,
            include_cleared: req.filter.map(|f| f.include_cleared).unwrap_or(false),
            page_size: req.pagination.as_ref().map(|p| p.page_size).unwrap_or(20),
            page_token: None,
        };

        let items = self.list_open_items_handler.handle(query).await
             .map_err(|e| Status::internal(e.to_string()))?;

        let proto_items = items.into_iter().map(|item| {
            OpenItem {
                document_reference: Some(common_v1::SystemDocumentReference {
                    document_number: item.document_number,
                    fiscal_year: item.fiscal_year,
                    company_code: item.company_code,
                    document_type: item.doc_type,
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

    // ----------------------------------------------------------------
    // Stub Implementations (Same as AP Service)
    // ----------------------------------------------------------------
    async fn get_account_balance(&self, request: Request<GetAccountBalanceRequest>) -> Result<Response<GetAccountBalanceResponse>, Status> {
        let req = request.into_inner();

        // List all open items for the customer
        let query = ListOpenItemsQuery {
            customer_id: req.business_partner_id.clone(),
            company_code: req.company_code.clone(),
            include_cleared: false,
            page_size: 1000,
            page_token: None,
        };

        let items = self.list_open_items_handler.handle(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

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

    async fn get_aging_analysis(&self, request: Request<GetAgingAnalysisRequest>) -> Result<Response<GetAgingAnalysisResponse>, Status> {
        let req = request.into_inner();

        // List all open items
        let query = ListOpenItemsQuery {
            customer_id: req.business_partner_id.clone(),
            company_code: req.company_code.clone(),
            include_cleared: false,
            page_size: 1000,
            page_token: None,
        };

        let items = self.list_open_items_handler.handle(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

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
    async fn post_invoice(&self, _r: Request<PostInvoiceRequest>) -> Result<Response<PostInvoiceResponse>, Status> { Err(Status::unimplemented("Handled by AP Service")) }
    
    async fn post_sales_invoice(
        &self,
        request: Request<PostSalesInvoiceRequest>,
    ) -> Result<Response<PostSalesInvoiceResponse>, Status> {
        let req = request.into_inner();

        // Default dates if not provided by Proto
        let now = chrono::Utc::now().date_naive();
        let currency = req.items.first()
            .and_then(|i| i.amount.as_ref())
            .map(|a| a.currency_code.clone())
            .unwrap_or_else(|| "CNY".to_string());

        let cmd = crate::application::commands::PostSalesInvoiceCommand {
            company_code: req.company_code.clone(),
            customer_id: req.customer_id,
            document_date: now,
            posting_date: now,
            currency,
            reference_document: None,
            header_text: None,
            items: req.items.into_iter().map(|item| {
                crate::application::commands::SalesInvoiceItemCommand {
                    gl_account: item.gl_account,
                    debit_credit: item.debit_credit_indicator,
                    amount: rust_decimal::Decimal::from_str(&item.amount.unwrap_or_default().value).unwrap_or_default(),
                    cost_center: if item.cost_center.is_empty() { None } else { Some(item.cost_center) },
                    item_text: if item.item_text.is_empty() { None } else { Some(item.item_text) },
                }
            }).collect(),
        };

        let invoice = self.post_sales_invoice_handler.handle(cmd).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(PostSalesInvoiceResponse {
            document: Some(common_v1::SystemDocumentReference {
                document_number: invoice.document_number.unwrap_or_default(),
                fiscal_year: invoice.fiscal_year,
                company_code: invoice.company_code,
                document_type: "DR".to_string(), // Customer Invoice
                document_category: "".to_string(),
            }),
        }))
    }
    async fn reverse_document(&self, request: Request<ReverseDocumentRequest>) -> Result<Response<ReverseDocumentResponse>, Status> {
        let req = request.into_inner();

        let doc_ref = req.document_to_reverse
            .ok_or_else(|| Status::invalid_argument("Missing document reference"))?;

        // For MVP: mark document as reversed
        // Full implementation would:
        // 1. Find the original sales invoice
        // 2. Create reversal GL entry via GL service
        // 3. Create reversal open items
        // 4. Update original invoice status to REVERSED

        // Simplified implementation - just acknowledge the request
        Ok(Response::new(ReverseDocumentResponse {
            success: true,
        }))
    }
    async fn verify_invoice(&self, _r: Request<VerifyInvoiceRequest>) -> Result<Response<VerifyInvoiceResponse>, Status> { Err(Status::unimplemented("")) }
    async fn generate_statement(&self, request: Request<GenerateStatementRequest>) -> Result<Response<GenerateStatementResponse>, Status> {
        let req = request.into_inner();

        // Get all open items for the customer (both cleared and uncleared)
        let query = ListOpenItemsQuery {
            customer_id: req.business_partner_id.clone(),
            company_code: req.company_code.clone(),
            include_cleared: true, // Include all items for statement
            page_size: 1000,
            page_token: None,
        };

        let items = self.list_open_items_handler.handle(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Sort by posting date
        let mut sorted_items = items;
        sorted_items.sort_by(|a, b| a.posting_date.cmp(&b.posting_date));

        // Create statement items with running balance
        let mut running_balance = rust_decimal::Decimal::ZERO;
        let mut statement_items = Vec::new();
        let mut currency = "CNY".to_string();

        for item in sorted_items {
            currency = item.currency.clone();
            running_balance += item.original_amount;

            statement_items.push(ap_v1::StatementItem {
                posting_date: Some(prost_types::Timestamp {
                    seconds: item.posting_date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                    nanos: 0,
                }),
                document_type_desc: item.doc_type.clone(),
                reference: item.reference_document.unwrap_or_else(|| item.document_number.clone()),
                amount: Some(common_v1::MonetaryValue {
                    value: item.original_amount.to_string(),
                    currency_code: currency.clone(),
                }),
                open_balance: Some(common_v1::MonetaryValue {
                    value: running_balance.to_string(),
                    currency_code: currency.clone(),
                }),
            });
        }

        Ok(Response::new(GenerateStatementResponse {
            items: statement_items,
            closing_balance: Some(common_v1::MonetaryValue {
                value: running_balance.to_string(),
                currency_code: currency,
            }),
        }))
    }
    async fn get_dunning_history(&self, _r: Request<GetDunningHistoryRequest>) -> Result<Response<GetDunningHistoryResponse>, Status> { Err(Status::unimplemented("")) }
    async fn trigger_dunning(&self, _r: Request<TriggerDunningRequest>) -> Result<Response<TriggerDunningResponse>, Status> { Err(Status::unimplemented("")) }
    async fn get_clearing_proposal(&self, request: Request<GetClearingProposalRequest>) -> Result<Response<GetClearingProposalResponse>, Status> {
        let req = request.into_inner();

        // Get all open items for the customer
        let query = ListOpenItemsQuery {
            customer_id: req.business_partner_id.clone(),
            company_code: req.company_code.clone(),
            include_cleared: false,
            page_size: 1000,
            page_token: None,
        };

        let items = self.list_open_items_handler.handle(query).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Separate debit and credit items
        let mut debit_items = Vec::new();
        let mut credit_items = Vec::new();

        for item in items {
            let identifier = ap_v1::OpenItemIdentifier {
                document_number: item.document_number.clone(),
                fiscal_year: item.fiscal_year,
                line_item_number: item.line_item_number,
            };

            // For AR: Positive amounts are typically debit (customer invoices)
            // Negative amounts are typically credit (payments, credit memos)
            if item.open_amount >= rust_decimal::Decimal::ZERO {
                debit_items.push((item.open_amount, identifier, item.currency.clone()));
            } else {
                credit_items.push((item.open_amount.abs(), identifier, item.currency.clone()));
            }
        }

        // Simple matching algorithm: match items with equal amounts
        let mut proposals = Vec::new();
        let mut used_debit_indices = std::collections::HashSet::new();
        let mut used_credit_indices = std::collections::HashSet::new();

        for (di, (d_amount, d_id, d_curr)) in debit_items.iter().enumerate() {
            if used_debit_indices.contains(&di) {
                continue;
            }

            for (ci, (c_amount, c_id, c_curr)) in credit_items.iter().enumerate() {
                if used_credit_indices.contains(&ci) {
                    continue;
                }

                // Match if amounts are equal and currency matches
                if d_amount == c_amount && d_curr == c_curr {
                    proposals.push(ap_v1::ClearingProposalMatch {
                        debit_items: vec![d_id.clone()],
                        credit_items: vec![c_id.clone()],
                        match_amount: Some(common_v1::MonetaryValue {
                            value: d_amount.to_string(),
                            currency_code: d_curr.clone(),
                        }),
                        match_score: 1.0, // Perfect match
                    });

                    used_debit_indices.insert(di);
                    used_credit_indices.insert(ci);
                    break;
                }
            }
        }

        Ok(Response::new(GetClearingProposalResponse {
            proposals,
        }))
    }
    async fn execute_clearing_proposal(&self, request: Request<ExecuteClearingProposalRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> {
        let req = request.into_inner();

        // For MVP: acknowledge the request
        // Full implementation would:
        // 1. Retrieve the proposals from the previous get_clearing_proposal call
        // 2. Execute clearing for the selected proposal indices
        // 3. Create clearing documents
        // 4. Update open items

        Ok(Response::new(ClearOpenItemsResponse {
            success: true,
            clearing_document: None,
        }))
    }
    async fn clear_open_items(&self, request: Request<ClearOpenItemsRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> {
        let _req = request.into_inner();

        // Simplified stub - full implementation requires proper ID extraction
        Ok(Response::new(ClearOpenItemsResponse {
            success: true,
            clearing_document: None,
        }))
    }
    async fn partial_clear_items(&self, request: Request<PartialClearItemsRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> {
        let _req = request.into_inner();

        // Simplified stub - full implementation requires item lookup
        Ok(Response::new(ClearOpenItemsResponse {
            success: true,
            clearing_document: None,
        }))
    }
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
