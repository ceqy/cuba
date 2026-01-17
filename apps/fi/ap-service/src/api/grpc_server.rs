//! AP gRPC Service Implementation

use tonic::{Request, Response, Status};
use std::sync::Arc;

use crate::application::commands::{PostSupplierCommand, ListOpenItemsQuery};
use crate::application::handlers::{PostSupplierHandler, ListOpenItemsHandler, PostInvoiceHandler, GetInvoiceHandler, ApproveInvoiceHandler, RejectInvoiceHandler, ClearOpenItemsHandler, PartialClearHandler, GeneratePaymentProposalHandler, ExecutePaymentProposalHandler};

// Use the properly structured proto modules
use crate::api::proto::fi::ap::v1 as ap_v1;
use crate::api::proto::common::v1 as common_v1;

use ap_v1::accounts_receivable_payable_service_server::AccountsReceivablePayableService;
use ap_v1::*;

use chrono::Datelike;
use std::str::FromStr;

/// gRPC Service Implementation
#[allow(dead_code)]
pub struct ApServiceImpl {
    post_supplier_handler: Arc<PostSupplierHandler>,
    list_open_items_handler: Arc<ListOpenItemsHandler>,
    post_invoice_handler: Arc<PostInvoiceHandler>,
    get_invoice_handler: Arc<GetInvoiceHandler>,
    approve_invoice_handler: Arc<ApproveInvoiceHandler>,
    reject_invoice_handler: Arc<RejectInvoiceHandler>,
    clear_open_items_handler: Arc<ClearOpenItemsHandler>,
    partial_clear_handler: Arc<PartialClearHandler>,
    generate_payment_proposal_handler: Arc<GeneratePaymentProposalHandler>,
    execute_payment_proposal_handler: Arc<ExecutePaymentProposalHandler>,
}

impl ApServiceImpl {
    pub fn new(
        post_supplier_handler: Arc<PostSupplierHandler>,
        list_open_items_handler: Arc<ListOpenItemsHandler>,
        post_invoice_handler: Arc<PostInvoiceHandler>,
        get_invoice_handler: Arc<GetInvoiceHandler>,
        approve_invoice_handler: Arc<ApproveInvoiceHandler>,
        reject_invoice_handler: Arc<RejectInvoiceHandler>,
        clear_open_items_handler: Arc<ClearOpenItemsHandler>,
        partial_clear_handler: Arc<PartialClearHandler>,
        generate_payment_proposal_handler: Arc<GeneratePaymentProposalHandler>,
        execute_payment_proposal_handler: Arc<ExecutePaymentProposalHandler>,
    ) -> Self {
        Self {
            post_supplier_handler,
            list_open_items_handler,
            post_invoice_handler,
            get_invoice_handler,
            approve_invoice_handler,
            reject_invoice_handler,
            clear_open_items_handler,
            partial_clear_handler,
            generate_payment_proposal_handler,
            execute_payment_proposal_handler,
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
        request: Request<ReverseDocumentRequest>,
    ) -> Result<Response<ReverseDocumentResponse>, Status> {
        let req = request.into_inner();

        let _doc_ref = req.document_to_reverse
            .ok_or_else(|| Status::invalid_argument("Missing document reference"))?;

        // For MVP: mark document as reversed
        // Full implementation would:
        // 1. Find the original invoice
        // 2. Create reversal GL entry via GL service
        // 3. Create reversal open items
        // 4. Update original invoice status to REVERSED

        // Simplified implementation - just acknowledge the request
        Ok(Response::new(ReverseDocumentResponse {
            success: true,
        }))
    }

    async fn verify_invoice(
        &self,
        request: Request<VerifyInvoiceRequest>,
    ) -> Result<Response<VerifyInvoiceResponse>, Status> {
        let req = request.into_inner();

        // Get document reference
        let _doc_ref = req.document.ok_or_else(|| Status::invalid_argument("Missing document reference"))?;

        // For simplicity, we'll just return success
        // In real implementation, this would validate against purchase orders, goods receipts, etc.
        Ok(Response::new(VerifyInvoiceResponse {
            success: true,
        }))
    }


    // ----------------------------------------------------------------
    // Payments & Clearing (Stubs)
    // ----------------------------------------------------------------

    async fn generate_statement(&self, request: Request<GenerateStatementRequest>) -> Result<Response<GenerateStatementResponse>, Status> {
        let req = request.into_inner();

        // Get all open items for the supplier (both cleared and uncleared)
        let query = ListOpenItemsQuery {
            business_partner_id: req.business_partner_id.clone(),
            company_code: req.company_code.clone(),
            account_type: "K".to_string(), // K for vendor
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
                document_type_desc: item.account_type.clone(),
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
    async fn get_dunning_history(&self, request: Request<GetDunningHistoryRequest>) -> Result<Response<GetDunningHistoryResponse>, Status> {
        let _req = request.into_inner();

        // For MVP: return mock dunning history
        // Full implementation would:
        // 1. Query dunning documents for the business partner
        // 2. Return dunning history with dunning levels and dates
        // 3. Support multiple dunning runs

        // Mock dunning records with 3 levels
        let history = vec![
            ap_v1::DunningRecord {
                dunning_date: Some(prost_types::Timestamp {
                    seconds: chrono::Utc::now().timestamp() - 30 * 86400, // 30 days ago
                    nanos: 0,
                }),
                dunning_level: 1,
                dunning_amount: Some(common_v1::MonetaryValue {
                    value: "5000.00".to_string(),
                    currency_code: "CNY".to_string(),
                }),
                dunning_text: "First dunning notice".to_string(),
            },
            ap_v1::DunningRecord {
                dunning_date: Some(prost_types::Timestamp {
                    seconds: chrono::Utc::now().timestamp() - 15 * 86400, // 15 days ago
                    nanos: 0,
                }),
                dunning_level: 2,
                dunning_amount: Some(common_v1::MonetaryValue {
                    value: "5000.00".to_string(),
                    currency_code: "CNY".to_string(),
                }),
                dunning_text: "Second dunning notice".to_string(),
            },
        ];

        Ok(Response::new(GetDunningHistoryResponse {
            history,
        }))
    }
    async fn trigger_dunning(&self, request: Request<TriggerDunningRequest>) -> Result<Response<TriggerDunningResponse>, Status> {
        let req = request.into_inner();

        // For MVP: acknowledge the dunning trigger
        // Full implementation would:
        // 1. Query all overdue open items
        // 2. Determine dunning level based on days overdue
        // 3. Create dunning documents (DUNNING NOTICE)
        // 4. Update open items with dunning information
        // 5. Send notifications if configured

        // Validate dunning_date if provided
        if let Some(_ts) = req.dunning_date {
            // Validate that date is reasonable
        }

        Ok(Response::new(TriggerDunningResponse {
            success: true,
        }))
    }
    async fn get_clearing_proposal(&self, request: Request<GetClearingProposalRequest>) -> Result<Response<GetClearingProposalResponse>, Status> {
        let req = request.into_inner();

        // Get all open items for the business partner
        let query = ListOpenItemsQuery {
            business_partner_id: req.business_partner_id.clone(),
            company_code: req.company_code.clone(),
            account_type: "K".to_string(), // K for vendor
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

            // Determine if debit or credit based on amount sign
            // Positive amounts are typically debit (supplier invoices)
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
        let _req = request.into_inner();

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

        // Simplified stub - full implementation requires mapping OpenItemIdentifier
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
    async fn net_clearing(&self, _r: Request<NetClearingRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> {
        // For MVP: simplified stub
        // Full implementation would:
        // 1. Query all open items for multiple suppliers
        // 2. Perform bilateral netting (receivables - payables)
        // 3. Create net clearing documents
        // 4. Update GL for net positions

        Ok(Response::new(ClearOpenItemsResponse {
            success: true,
            clearing_document: None,
        }))
    }
    async fn check_credit_limit(&self, request: Request<CheckCreditLimitRequest>) -> Result<Response<CheckCreditLimitResponse>, Status> {
        let req = request.into_inner();

        // For MVP: simplified credit check
        // Full implementation would:
        // 1. Query customer/supplier master data for credit limit
        // 2. Query open items and sum them up
        // 3. Compare used credit against limit
        // 4. Apply any approved credit increases/holds

        // Parse the amount from the request
        let requested_amount = if let Some(amount) = req.amount {
            rust_decimal::Decimal::from_str(&amount.value).unwrap_or(rust_decimal::Decimal::ZERO)
        } else {
            rust_decimal::Decimal::ZERO
        };

        // Mock credit limit check: Assume 100,000 CNY total limit
        let total_limit = rust_decimal::Decimal::from_str("100000.00").unwrap();
        let used_credit = rust_decimal::Decimal::from_str("30000.00").unwrap();
        let available_credit = total_limit - used_credit;

        let passed = requested_amount <= available_credit;

        Ok(Response::new(CheckCreditLimitResponse {
            result: Some(ap_v1::CreditCheckResult {
                used_credit: Some(common_v1::MonetaryValue {
                    value: used_credit.to_string(),
                    currency_code: "CNY".to_string(),
                }),
                total_limit: Some(common_v1::MonetaryValue {
                    value: total_limit.to_string(),
                    currency_code: "CNY".to_string(),
                }),
                passed,
                block_reason: if !passed {
                    format!("Credit limit exceeded. Available: {}, Requested: {}", available_credit, requested_amount)
                } else {
                    String::new()
                },
            }),
        }))
    }
    async fn update_credit_exposure(&self, request: Request<UpdateCreditExposureRequest>) -> Result<Response<UpdateCreditExposureResponse>, Status> {
        let req = request.into_inner();

        // For MVP: acknowledge the credit exposure update
        // Full implementation would:
        // 1. Update the business partner's credit limit
        // 2. Create an audit trail entry
        // 3. Validate amount is reasonable
        // 4. Notify if credit limit increased/decreased significantly

        if let Some(_amount) = req.amount {
            // In a real system, we would update the customer/supplier master data
            // and create an audit entry
        }

        Ok(Response::new(UpdateCreditExposureResponse {
            success: true,
        }))
    }
    async fn generate_payment_proposal(&self, request: Request<GeneratePaymentProposalRequest>) -> Result<Response<GeneratePaymentProposalResponse>, Status> {
        let req = request.into_inner();

        // Convert due_date timestamp to NaiveDate
        let due_date = if let Some(ts) = req.due_date {
            chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + chrono::Duration::days(ts.seconds / 86400)
        } else {
            chrono::Utc::now().naive_utc().date()
        };

        // Get due items from handler
        let open_items = self.generate_payment_proposal_handler.handle(req.company_code, due_date).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Group by supplier
        let mut supplier_map: std::collections::HashMap<String, (rust_decimal::Decimal, Vec<ap_v1::OpenItemIdentifier>)> = std::collections::HashMap::new();

        for item in open_items {
            let supplier_id = item.supplier_id.map(|id| id.to_string()).unwrap_or_default();
            if supplier_id.is_empty() {
                continue;
            }

            let entry = supplier_map.entry(supplier_id.clone()).or_insert((rust_decimal::Decimal::ZERO, vec![]));
            entry.0 += item.open_amount;
            entry.1.push(ap_v1::OpenItemIdentifier {
                document_number: item.document_number,
                fiscal_year: item.fiscal_year,
                line_item_number: item.line_item_number,
            });
        }

        // Convert to PaymentProposalItem
        let items: Vec<ap_v1::PaymentProposalItem> = supplier_map.into_iter().map(|(supplier_id, (total_amount, open_items))| {
            ap_v1::PaymentProposalItem {
                supplier_id,
                proposed_amount: Some(common_v1::MonetaryValue {
                    value: total_amount.to_string(),
                    currency_code: "CNY".to_string(), // Default currency
                }),
                open_items,
                blocked: false,
            }
        }).collect();

        Ok(Response::new(GeneratePaymentProposalResponse {
            items,
        }))
    }

    async fn execute_payment_proposal(&self, request: Request<ExecutePaymentProposalRequest>) -> Result<Response<PaymentExecutionResponse>, Status> {
        let _req = request.into_inner();

        // For each supplier, get their open items and create a payment document
        // In a real implementation, this would:
        // 1. Create a payment document in the system
        // 2. Clear all the open items
        // 3. Create GL entries for the payment
        // 4. Integrate with payment systems

        // For MVP: just acknowledge the request
        // Full implementation would query open items for each supplier and clear them

        Ok(Response::new(PaymentExecutionResponse {
            success: true,
        }))
    }
    async fn request_down_payment(&self, request: Request<DownPaymentRequest>) -> Result<Response<DownPaymentResponse>, Status> {
        let req = request.into_inner();

        // For MVP: create a down payment document
        // Full implementation would:
        // 1. Validate supplier account exists
        // 2. Create a down payment advance (APP - Advance Payment)
        // 3. Create GL entry for cash (debit) and payable (credit)
        // 4. Link to purchase orders if provided

        let dp_doc_number = format!("APP-{}-{}",
            chrono::Utc::now().format("%Y%m%d"),
            uuid::Uuid::new_v4().simple().to_string().chars().take(6).collect::<String>()
        );

        Ok(Response::new(DownPaymentResponse {
            document: Some(common_v1::SystemDocumentReference {
                document_number: dp_doc_number,
                fiscal_year: chrono::Utc::now().year(),
                company_code: req.company_code,
                document_type: "APP".to_string(),
                document_category: "DOWN_PAYMENT".to_string(),
            }),
        }))
    }
    async fn clear_down_payment(&self, request: Request<DownPaymentClearingRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> {
        let _req = request.into_inner();

        // For MVP: acknowledge down payment clearing
        // Full implementation would:
        // 1. Match down payment item with invoice
        // 2. Reduce invoice amount by down payment
        // 3. Clear the down payment open item
        // 4. Create GL entries for the adjustment

        Ok(Response::new(ClearOpenItemsResponse {
            success: true,
            clearing_document: None,
        }))
    }
    async fn list_attachments(&self, request: Request<ListAttachmentsRequest>) -> Result<Response<ListAttachmentsResponse>, Status> {
        let req = request.into_inner();

        // For MVP: return mock attachments
        // Full implementation would:
        // 1. Query attachment metadata from database or S3/MinIO
        // 2. Support filters like file_type, date_range, uploaded_by
        // 3. Return attachment list with metadata (size, upload_date, etc.)
        // 4. Integrate with object storage for actual file management

        // Mock attachments for the document
        let attachments = vec![
            ap_v1::list_attachments_response::AttachmentMetadata {
                attachment_id: format!("{}-001", req.document_number),
                file_name: format!("invoice_{}.pdf", req.document_number),
                file_type: "application/pdf".to_string(),
                file_size: 245632,
                uploaded_at: Some(prost_types::Timestamp {
                    seconds: chrono::Utc::now().timestamp() - 7 * 86400, // 7 days ago
                    nanos: 0,
                }),
                uploaded_by: "system".to_string(),
            },
            ap_v1::list_attachments_response::AttachmentMetadata {
                attachment_id: format!("{}-002", req.document_number),
                file_name: format!("po_receipt_{}.pdf", req.document_number),
                file_type: "application/pdf".to_string(),
                file_size: 128456,
                uploaded_at: Some(prost_types::Timestamp {
                    seconds: chrono::Utc::now().timestamp() - 5 * 86400, // 5 days ago
                    nanos: 0,
                }),
                uploaded_by: "warehouse_team".to_string(),
            },
        ];

        Ok(Response::new(ListAttachmentsResponse {
            attachments,
        }))
    }
    async fn upload_attachment(&self, _r: Request<UploadAttachmentRequest>) -> Result<Response<OperationResponse>, Status> { Err(Status::unimplemented("")) }
    async fn import_bank_statement(&self, request: Request<ImportBankStatementRequest>) -> Result<Response<ImportBankStatementResponse>, Status> {
        let req = request.into_inner();

        // For MVP: acknowledge bank statement import
        // Full implementation would:
        // 1. Parse SWIFT MT940 or proprietary bank format
        // 2. Create bank statement header
        // 3. Validate bank account and currency
        // 4. Create individual payment line items
        // 5. Match against open items for auto-clearing
        // 6. Create GL entries for cash movements

        if let Some(_stmt) = req.statement {
            // Mock: Process the statement
            // In reality, would parse the lines and create documents
        }

        Ok(Response::new(ImportBankStatementResponse {
            success: true,
        }))
    }
    async fn process_lockbox(&self, _r: Request<ProcessLockboxRequest>) -> Result<Response<ProcessLockboxResponse>, Status> {
        // For MVP: acknowledge lockbox processing
        // Full implementation would:
        // 1. Parse check images and metadata from bank lockbox
        // 2. Extract customer information and amount
        // 3. Create payment records
        // 4. Apply cash to customer accounts

        Err(Status::unimplemented("Lockbox processing requires image OCR"))
    }
    async fn apply_cash(&self, request: Request<ApplyCashRequest>) -> Result<Response<ClearOpenItemsResponse>, Status> {
        let _req = request.into_inner();

        // For MVP: acknowledge cash application
        // Full implementation would:
        // 1. Query open items for the company
        // 2. Apply cash to oldest items first (FIFO)
        // 3. Support partial payments and residual items
        // 4. Create clearing documents
        // 5. Create GL entries for cash receipt

        Ok(Response::new(ClearOpenItemsResponse {
            success: true,
            clearing_document: None,
        }))
    }
    async fn get_tolerance_groups(&self, request: Request<GetToleranceGroupsRequest>) -> Result<Response<GetToleranceGroupsResponse>, Status> {
        let _req = request.into_inner();

        // For MVP: return mock tolerance groups
        // Full implementation would:
        // 1. Query tolerance group master data
        // 2. Return percentage and amount tolerances
        // 3. Support tolerance for invoice verification (3-way match)
        // 4. Support variance limits for price/quantity/date

        let groups = vec![
            ap_v1::ToleranceGroup {
                id: "TOL001".to_string(),
            },
            ap_v1::ToleranceGroup {
                id: "TOL002".to_string(),
            },
            ap_v1::ToleranceGroup {
                id: "TOL003".to_string(),
            },
        ];

        Ok(Response::new(GetToleranceGroupsResponse {
            groups,
        }))
    }
    async fn perform_compliance_check(&self, request: Request<PerformComplianceCheckRequest>) -> Result<Response<PerformComplianceCheckResponse>, Status> {
        let _req = request.into_inner();

        // For MVP: return mock compliance check result
        // Full implementation would:
        // 1. Check sanctions lists (OFAC, EU, UN)
        // 2. Verify tax/business registration numbers
        // 3. Check for duplicate invoices
        // 4. Verify payment terms compliance
        // 5. Check for blocked suppliers/customers

        Ok(Response::new(PerformComplianceCheckResponse {
            passed: true,
        }))
    }
    async fn export_report(&self, request: Request<ExportReportRequest>) -> Result<Response<ExportReportResponse>, Status> {
        let _req = request.into_inner();

        // For MVP: mock report export
        // Full implementation would:
        // 1. Support multiple formats: PDF, Excel, CSV
        // 2. Generate account statements, aging reports, GL reports
        // 3. Support date range and filtering
        // 4. Generate file and return download URL
        // 5. Archive reports for audit trail

        Ok(Response::new(ExportReportResponse {
            download_url: "s3://reports/ap-aging-2024-01-18.pdf".to_string(),
        }))
    }
    async fn subscribe_to_events(&self, _r: Request<SubscribeToEventsRequest>) -> Result<Response<SubscribeToEventsResponse>, Status> {
        // For MVP: not implemented
        // Full implementation would:
        // 1. Support subscribe/unsubscribe to business events
        // 2. Events: invoice_posted, payment_executed, account_cleared
        // 3. Integration with message queue (Kafka/RabbitMQ)
        // 4. Webhook support for external systems

        Err(Status::unimplemented("Event subscription requires message queue infrastructure"))
    }
    async fn list_event_types(&self, _r: Request<ListEventTypesRequest>) -> Result<Response<ListEventTypesResponse>, Status> {
        // For MVP: return available event types
        // Full implementation would query from configuration

        let types = vec![
            ap_v1::EventType {
                event_code: "INVOICE_POSTED".to_string(),
                description: "Invoice has been posted to GL".to_string(),
            },
            ap_v1::EventType {
                event_code: "PAYMENT_EXECUTED".to_string(),
                description: "Payment has been executed".to_string(),
            },
            ap_v1::EventType {
                event_code: "DOCUMENT_CLEARED".to_string(),
                description: "Document has been cleared".to_string(),
            },
            ap_v1::EventType {
                event_code: "DUNNING_TRIGGERED".to_string(),
                description: "Dunning notice has been sent".to_string(),
            },
        ];

        Ok(Response::new(ListEventTypesResponse {
            types,
        }))
    }
    async fn post_sales_invoice(&self, _r: Request<PostSalesInvoiceRequest>) -> Result<Response<PostSalesInvoiceResponse>, Status> { Err(Status::unimplemented("Handled by AR Service")) }
}
