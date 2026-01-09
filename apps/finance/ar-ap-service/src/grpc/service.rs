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
    
    // TODO: Implement remaining 33 RPC methods in future phases
}
