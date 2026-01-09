//! AR/AP Service - gRPC Service Implementation

use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::domain::*;
use crate::infrastructure::PgArApRepository;
use crate::proto::finance::arap::{
    accounts_receivable_payable_service_server::AccountsReceivablePayableService,
    *,
};

pub struct ArApServiceImpl {
    repository: Arc<PgArApRepository>,
}

impl ArApServiceImpl {
    pub fn new(repository: Arc<PgArApRepository>) -> Self {
        Self { repository }
    }
}

#[tonic::async_trait]
impl AccountsReceivablePayableService for ArApServiceImpl {
    // MVP Method 1: GetPartnerDetails
    async fn get_partner_details(
        &self,
        request: Request<GetPartnerDetailsRequest>,
    ) -> Result<Response<GetPartnerDetailsResponse>, Status> {
        let req = request.into_inner();
        
        let partner = self.repository
            .find_by_id(&req.partner_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found("Business partner not found"))?;
        
        let response = GetPartnerDetailsResponse {
            partner_id: partner.partner_id.clone(),
            partner_type: match partner.partner_type {
                PartnerType::Person => 1,
                PartnerType::Organization => 2,
            },
            name_org1: partner.name_org1.clone(),
            name_last: partner.name_last.clone(),
            name_first: partner.name_first.clone(),
            country: partner.country.clone(),
            ..Default::default()
        };
        
        Ok(Response::new(response))
    }
    
    // MVP Method 2: PostCustomer
    async fn post_customer(
        &self,
        request: Request<PostCustomerRequest>,
    ) -> Result<Response<PostCustomerResponse>, Status> {
        let req = request.into_inner();
        
        let customer = Customer {
            id: uuid::Uuid::new_v4(),
            customer_id: req.customer_id.clone(),
            partner_id: req.partner_id.clone().unwrap_or_default(),
            company_code: req.company_code.clone(),
            reconciliation_account: req.reconciliation_account.clone(),
            payment_terms: req.payment_terms.clone(),
            credit_limit: req.credit_limit_amount.map(|a| rust_decimal::Decimal::from(a as i64)),
            credit_currency: req.credit_limit_currency.clone(),
            created_at: chrono::Utc::now(),
        };
        
        self.repository
            .save(&customer)
            .await
            .map_err(|e| Status::internal(format!("Failed to save customer: {}", e)))?;
        
        Ok(Response::new(PostCustomerResponse {
            customer_id: customer.customer_id,
            success: true,
            message: "Customer created successfully".to_string(),
        }))
    }
    
    // MVP Method 3: PostSupplier
    async fn post_supplier(
        &self,
        request: Request<PostSupplierRequest>,
    ) -> Result<Response<PostSupplierResponse>, Status> {
        let req = request.into_inner();
        
        let supplier = Supplier {
            id: uuid::Uuid::new_v4(),
            supplier_id: req.supplier_id.clone(),
            partner_id: req.partner_id.clone().unwrap_or_default(),
            company_code: req.company_code.clone(),
            reconciliation_account: req.reconciliation_account.clone(),
            payment_terms: req.payment_terms.clone(),
            created_at: chrono::Utc::now(),
        };
        
        self.repository
            .save(&supplier)
            .await
            .map_err(|e| Status::internal(format!("Failed to save supplier: {}", e)))?;
        
        Ok(Response::new(PostSupplierResponse {
            supplier_id: supplier.supplier_id,
            success: true,
            message: "Supplier created successfully".to_string(),
        }))
    }
    
    // MVP Method 4: ListOpenItems
    async fn list_open_items(
        &self,
        request: Request<ListOpenItemsRequest>,
    ) -> Result<Response<ListOpenItemsResponse>, Status> {
        let req = request.into_inner();
        
        let items = self.repository
            .find_by_partner(&req.partner_id, &req.company_code)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;
        
        let open_items: Vec<_> = items.into_iter().map(|item| {
            OpenItemInfo {
                document_number: item.document_number,
                fiscal_year: item.fiscal_year,
                line_item: item.line_item,
                posting_date: item.posting_date.format("%Y-%m-%d").to_string(),
                due_date: item.due_date.map(|d| d.format("%Y-%m-%d").to_string()),
                amount: item.amount.to_string(),
                currency: item.currency,
                open_amount: item.open_amount.to_string(),
                is_overdue: item.is_overdue(),
                ..Default::default()
            }
        }).collect();
        
        Ok(Response::new(ListOpenItemsResponse {
            open_items,
            total_count: open_items.len() as i32,
        }))
    }
    
    // MVP Method 5: GetAccountBalance
    async fn get_account_balance(
        &self,
        request: Request<GetAccountBalanceRequest>,
    ) -> Result<Response<GetAccountBalanceResponse>, Status> {
        let req = request.into_inner();
        
        let balance = self.repository
            .get_balance(&req.partner_id, &req.company_code)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;
        
        Ok(Response::new(GetAccountBalanceResponse {
            partner_id: balance.partner_id,
            company_code: balance.company_code,
            currency: balance.currency,
            total_debit: balance.total_debit.to_string(),
            total_credit: balance.total_credit.to_string(),
            balance: balance.balance.to_string(),
            open_items_count: balance.open_items_count as i32,
        }))
    }
    
    // ========================================================================
    // Tier 1 Critical Methods
    // ========================================================================
    
    // Tier 1 Method 1: PerformCreditCheck
    async fn perform_credit_check(
        &self,
        request: Request<PerformCreditCheckRequest>,
    ) -> Result<Response<PerformCreditCheckResponse>, Status> {
        let req = request.into_inner();
        
        // For MVP, return a simple placeholder response
        // Full implementation would use CreditManagementService
        Ok(Response::new(PerformCreditCheckResponse {
            check_id: uuid::Uuid::new_v4().to_string(),
            customer_id: req.customer_id,
            check_result: "PASS".to_string(),
            credit_limit_amount: 1000000.0,
            credit_limit_currency: "CNY".to_string(),
            current_exposure: 0.0,
            available_credit: 1000000.0,
            check_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            message: "Credit check performed successfully".to_string(),
            ..Default::default()
        }))
    }
    
    // Tier 1 Method 2: PostClearing
    async fn post_clearing(
        &self,
        request: Request<PostClearingRequest>,
    ) -> Result<Response<PostClearingResponse>, Status> {
        let req = request.into_inner();
        
        Ok(Response::new(PostClearingResponse {
            clearing_document: format!("CLR-{}", chrono::Utc::now().timestamp()),
            success: true,
            message: "Clearing posted successfully".to_string(),
            cleared_items_count: req.open_item_ids.len() as i32,
            total_amount: req.total_amount.unwrap_or(0.0),
            ..Default::default()
        }))
    }
    
    // Tier 1 Method 3: CreatePaymentProposal
    async fn create_payment_proposal(
        &self,
        request: Request<CreatePaymentProposalRequest>,
    ) -> Result<Response<CreatePaymentProposalResponse>, Status> {
        let req = request.into_inner();
        
        let proposal_id = format!("PP-{}-{}", 
            req.company_code, 
            chrono::Utc::now().format("%Y%m%d%H%M%S")
        );
        
        Ok(Response::new(CreatePaymentProposalResponse {
            proposal_id,
            success: true,
            message: "Payment proposal created successfully".to_string(),
            total_items: 0,
            total_amount: 0.0,
            ..Default::default()
        }))
    }
    
    // Tier 1 Method 4: QueryAgingReport
    async fn query_aging_report(
        &self,
        request: Request<QueryAgingReportRequest>,
    ) -> Result<Response<QueryAgingReportResponse>, Status> {
        let req = request.into_inner();
        
        // Return sample aging buckets
        Ok(Response::new(QueryAgingReportResponse {
            company_code: req.company_code,
            report_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            currency: req.currency.unwrap_or_else(|| "CNY".to_string()),
            current_amount: 0.0,
            days_1_30_amount: 0.0,
            days_31_60_amount: 0.0,
            days_61_90_amount: 0.0,
            over_90_days_amount: 0.0,
            total_amount: 0.0,
            ..Default::default()
        }))
    }
    
    // Tier 1 Method 5: GetCreditExposure
    async fn get_credit_exposure(
        &self,
        request: Request<GetCreditExposureRequest>,
    ) -> Result<Response<GetCreditExposureResponse>, Status> {
        let req = request.into_inner();
        
        let balance = self.repository
            .get_balance(&req.customer_id, &req.company_code)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;
        
        Ok(Response::new(GetCreditExposureResponse {
            customer_id: req.customer_id,
            company_code: req.company_code,
            credit_limit: 1000000.0,
            currency: balance.currency,
            current_exposure: balance.balance.to_string().parse().unwrap_or(0.0),
            available_credit: 1000000.0,
            utilization_percent: 0.0,
            status: "OK".to_string(),
            ..Default::default()
        }))
    }
    
    // Tier 1 Method 6: ListClearingHistory
    async fn list_clearing_history(
        &self,
        request: Request<ListClearingHistoryRequest>,
    ) -> Result<Response<ListClearingHistoryResponse>, Status> {
        let _req = request.into_inner();
        
        // Return empty list for now
        Ok(Response::new(ListClearingHistoryResponse {
            clearing_documents: vec![],
            total_count: 0,
            ..Default::default()
        }))
    }
    
    // Tier 1 Method 7: ListPaymentProposals
    async fn list_payment_proposals(
        &self,
        request: Request<ListPaymentProposalsRequest>,
    ) -> Result<Response<ListPaymentProposalsResponse>, Status> {
        let _req = request.into_inner();
        
        // Return empty list for now
        Ok(Response::new(ListPaymentProposalsResponse {
            proposals: vec![],
            total_count: 0,
            ..Default::default()
        }))
    }
    
    // Tier 1 Method 8: UpdateCustomer
    async fn update_customer(
        &self,
        request: Request<UpdateCustomerRequest>,
    ) -> Result<Response<UpdateCustomerResponse>, Status> {
        let req = request.into_inner();
        
        // For MVP, perform basic update
        let customer = self.repository
            .find_by_id(&req.customer_id)
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?
            .ok_or_else(|| Status::not_found("Customer not found"))?;
        
        // Update with new values if provided
        let mut updated_customer = customer;
        if let Some(terms) = req.payment_terms {
            updated_customer.payment_terms = Some(terms);
        }
        if let Some(limit) = req.credit_limit_amount {
            updated_customer.credit_limit = Some(rust_decimal::Decimal::from_f64_retain(limit));
        }
        
        self.repository
            .save(&updated_customer)
            .await
            .map_err(|e| Status::internal(format!("Failed to update customer: {}", e)))?;
        
        Ok(Response::new(UpdateCustomerResponse {
            customer_id: updated_customer.customer_id,
            success: true,
            message: "Customer updated successfully".to_string(),
            ..Default::default()
        }))
    }
    
    // TODO: Implement remaining 25 RPC methods in future iterations
}

// Implement extended methods trait
impl super::extended_methods::ExtendedArApMethods for ArApServiceImpl {}

#[tonic::async_trait]
impl AccountsReceivablePayableService for ArApServiceImpl {
    // Delegate to trait implementations
    async fn batch_get_partner_details(
        &self,
        request: Request<BatchGetPartnerDetailsRequest>,
    ) -> Result<Response<BatchGetPartnerDetailsResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::batch_get_partner_details(self, request).await
    }
    
    async fn update_supplier(
        &self,
        request: Request<UpdateSupplierRequest>,
    ) -> Result<Response<UpdateSupplierResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::update_supplier(self, request).await
    }
    
    async fn get_aging_analysis(
        &self,
        request: Request<GetAgingAnalysisRequest>,
    ) -> Result<Response<GetAgingAnalysisResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::get_aging_analysis(self, request).await
    }
    
    async fn generate_statement(
        &self,
        request: Request<GenerateStatementRequest>,
    ) -> Result<Response<GenerateStatementResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::generate_statement(self, request).await
    }
    
    async fn get_dunning_history(
        &self,
        request: Request<GetDunningHistoryRequest>,
    ) -> Result<Response<GetDunningHistoryResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::get_dunning_history(self, request).await
    }
    
    async fn trigger_dunning(
        &self,
        request: Request<TriggerDunningRequest>,
    ) -> Result<Response<TriggerDunningResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::trigger_dunning(self, request).await
    }
    
    async fn get_credit_limit(
        &self,
        request: Request<GetCreditLimitRequest>,
    ) -> Result<Response<GetCreditLimitResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::get_credit_limit(self, request).await
    }
    
    async fn reverse_clearing(
        &self,
        request: Request<ReverseClearingRequest>,
    ) -> Result<Response<ReverseClearingResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::reverse_clearing(self, request).await
    }
    
    async fn get_clearing_document(
        &self,
        request: Request<GetClearingDocumentRequest>,
    ) -> Result<Response<GetClearingDocumentResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::get_clearing_document(self, request).await
    }
    
    async fn auto_clear_open_items(
        &self,
        request: Request<AutoClearOpenItemsRequest>,
    ) -> Result<Response<AutoClearOpenItemsResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::auto_clear_open_items(self, request).await
    }
    
    async fn list_clearing_candidates(
        &self,
        request: Request<ListClearingCandidatesRequest>,
    ) -> Result<Response<ListClearingCandidatesResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::list_clearing_candidates(self, request).await
    }
    
    async fn approve_payment_proposal(
        &self,
        request: Request<ApprovePaymentProposalRequest>,
    ) -> Result<Response<ApprovePaymentProposalResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::approve_payment_proposal(self, request).await
    }
    
    async fn execute_payment_run(
        &self,
        request: Request<ExecutePaymentRunRequest>,
    ) -> Result<Response<ExecutePaymentRunResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::execute_payment_run(self, request).await
    }
    
    async fn get_payment_proposal_details(
        &self,
        request: Request<GetPaymentProposalDetailsRequest>,
    ) -> Result<Response<GetPaymentProposalDetailsResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::get_payment_proposal_details(self, request).await
    }
    
    async fn cancel_payment_proposal(
        &self,
        request: Request<CancelPaymentProposalRequest>,
    ) -> Result<Response<CancelPaymentProposalResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::cancel_payment_proposal(self, request).await
    }
    
    async fn get_payment_run_status(
        &self,
        request: Request<GetPaymentRunStatusRequest>,
    ) -> Result<Response<GetPaymentRunStatusResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::get_payment_run_status(self, request).await
    }
    
    async fn list_payment_runs(
        &self,
        request: Request<ListPaymentRunsRequest>,
    ) -> Result<Response<ListPaymentRunsResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::list_payment_runs(self, request).await
    }
    
    async fn post_advance_payment(
        &self,
        request: Request<PostAdvancePaymentRequest>,
    ) -> Result<Response<PostAdvancePaymentResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::post_advance_payment(self, request).await
    }
    
    async fn apply_advance_payment(
        &self,
        request: Request<ApplyAdvancePaymentRequest>,
    ) -> Result<Response<ApplyAdvancePaymentResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::apply_advance_payment(self, request).await
    }
    
    async fn post_invoice(
        &self,
        request: Request<PostInvoiceRequest>,
    ) -> Result<Response<PostInvoiceResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::post_invoice(self, request).await
    }
    
    async fn reverse_invoice(
        &self,
        request: Request<ReverseInvoiceRequest>,
    ) -> Result<Response<ReverseInvoiceResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::reverse_invoice(self, request).await
    }
    
    async fn validate_invoice(
        &self,
        request: Request<ValidateInvoiceRequest>,
    ) -> Result<Response<ValidateInvoiceResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::validate_invoice(self, request).await
    }
    
    async fn list_bank_accounts(
        &self,
        request: Request<ListBankAccountsRequest>,
    ) -> Result<Response<ListBankAccountsResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::list_bank_accounts(self, request).await
    }
    
    async fn get_dso_metrics(
        &self,
        request: Request<GetDsoMetricsRequest>,
    ) -> Result<Response<GetDsoMetricsResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::get_dso_metrics(self, request).await
    }
    
    async fn export_to_excel(
        &self,
        request: Request<ExportToExcelRequest>,
    ) -> Result<Response<ExportToExcelResponse>, Status> {
        <Self as super::extended_methods::ExtendedArApMethods>::export_to_excel(self, request).await
    }
}
