//! AR/AP Service - Application Services
//!
//! 应用服务层 - 协调领域对象和基础设施

use std::sync::Arc;
use uuid::Uuid;

use crate::domain::*;

// ============================================================================
// Partner Service
// ============================================================================

pub struct PartnerService {
    partner_repo: Arc<dyn BusinessPartnerRepository>,
    customer_repo: Arc<dyn CustomerRepository>,
    supplier_repo: Arc<dyn SupplierRepository>,
}

impl PartnerService {
    pub fn new(
        partner_repo: Arc<dyn BusinessPartnerRepository>,
        customer_repo: Arc<dyn CustomerRepository>,
        supplier_repo: Arc<dyn SupplierRepository>,
    ) -> Self {
        Self {
            partner_repo,
            customer_repo,
            supplier_repo,
        }
    }
    
    pub async fn get_partner_details(&self, partner_id: &str) -> RepositoryResult<Option<BusinessPartner>> {
        self.partner_repo.find_by_id(partner_id).await
    }
    
    pub async fn create_customer(&self, customer: Customer) -> RepositoryResult<String> {
        self.customer_repo.save(&customer).await?;
        Ok(customer.customer_id)
    }
    
    pub async fn create_supplier(&self, supplier: Supplier) -> RepositoryResult<String> {
        self.supplier_repo.save(&supplier).await?;
        Ok(supplier.supplier_id)
    }
    
    pub async fn list_customers_by_company(&self, company_code: &str) -> RepositoryResult<Vec<Customer>> {
        self.customer_repo.find_by_company(company_code).await
    }
    
    pub async fn list_suppliers_by_company(&self, company_code: &str) -> RepositoryResult<Vec<Supplier>> {
        self.supplier_repo.find_by_company(company_code).await
    }
}

// ============================================================================
// Open Items Service
// ============================================================================

pub struct OpenItemsService {
    open_items_repo: Arc<dyn OpenItemRepository>,
}

impl OpenItemsService {
    pub fn new(open_items_repo: Arc<dyn OpenItemRepository>) -> Self {
        Self { open_items_repo }
    }
    
    pub async fn list_open_items(&self, partner_id: &str, company_code: &str) -> RepositoryResult<Vec<OpenItem>> {
        self.open_items_repo.find_by_partner(partner_id, company_code).await
    }
    
    pub async fn get_account_balance(&self, partner_id: &str, company_code: &str) -> RepositoryResult<AccountBalance> {
        self.open_items_repo.get_balance(partner_id, company_code).await
    }
    
    pub async fn get_overdue_items(&self, partner_id: &str) -> RepositoryResult<Vec<OpenItem>> {
        let items = self.open_items_repo.find_open_items(partner_id).await?;
        Ok(items.into_iter().filter(|item| item.is_overdue()).collect())
    }
}

// ============================================================================
// Credit Management Service
// ============================================================================

pub struct CreditManagementService {
    credit_check_repo: Arc<dyn CreditCheckRepository>,
    customer_repo: Arc<dyn CustomerRepository>,
    open_items_repo: Arc<dyn OpenItemRepository>,
}

impl CreditManagementService {
    pub fn new(
        credit_check_repo: Arc<dyn CreditCheckRepository>,
        customer_repo: Arc<dyn CustomerRepository>,
        open_items_repo: Arc<dyn OpenItemRepository>,
    ) -> Self {
        Self {
            credit_check_repo,
            customer_repo,
            open_items_repo,
        }
    }
    
    pub async fn perform_credit_check(&self, customer_id: &str, checked_by: Uuid) -> RepositoryResult<CreditCheck> {
        // Get customer info
        let customer = self.customer_repo.find_by_id(customer_id).await?
            .ok_or_else(|| RepositoryError::NotFound(format!("Customer {}", customer_id)))?;
        
        // Get current exposure
        let balance = self.open_items_repo.get_balance(&customer.partner_id, &customer.company_code).await?;
        
        // Build credit check result
        let (check_result, check_reason) = if let Some(credit_limit) = customer.credit_limit {
            let limit_money = Money::new(credit_limit, Currency::from_code(customer.credit_currency.as_deref().unwrap_or("CNY")));
            let exposure_money = Money::new(balance.balance, Currency::from_code(&balance.currency));
            let available = Money::new(credit_limit - balance.balance, limit_money.currency);
            
            let result = if balance.balance > credit_limit {
                (CreditCheckResult::Fail, Some("Credit limit exceeded".to_string()))
            } else if balance.balance / credit_limit > rust_decimal::Decimal::from_f32_retain(0.9).unwrap() {
                (CreditCheckResult::Warning, Some("Credit utilization > 90%".to_string()))
            } else {
                (CreditCheckResult::Pass, None)
            };
            
            let check = CreditCheck {
                id: Uuid::new_v4(),
                customer_id: customer_id.to_string(),
                check_date: chrono::Utc::now().date_naive(),
                credit_limit: Some(limit_money),
                current_exposure: exposure_money,
                available_credit: available,
                check_result: result.0,
                check_reason: result.1,
                checked_by: Some(checked_by),
                created_at: chrono::Utc::now(),
            };
            
            self.credit_check_repo.save(&check).await?;
            Ok(check)
        } else {
            // No credit limit set
            let check = CreditCheck {
                id: Uuid::new_v4(),
                customer_id: customer_id.to_string(),
                check_date: chrono::Utc::now().date_naive(),
                credit_limit: None,
                current_exposure: Money::new(balance.balance, Currency::from_code(&balance.currency)),
                available_credit: Money::zero(Currency::from_code(&balance.currency)),
                check_result: CreditCheckResult::Warning,
                check_reason: Some("No credit limit set".to_string()),
                checked_by: Some(checked_by),
                created_at: chrono::Utc::now(),
            };
            
            self.credit_check_repo.save(&check).await?;
            Ok(check)
        }
    }
}

// ============================================================================
// Payment Service
// ============================================================================

pub struct PaymentService {
    payment_proposal_repo: Arc<dyn PaymentProposalRepository>,
    payment_run_repo: Arc<dyn PaymentRunRepository>,
    open_items_repo: Arc<dyn OpenItemRepository>,
}

impl PaymentService {
    pub fn new(
        payment_proposal_repo: Arc<dyn PaymentProposalRepository>,
        payment_run_repo: Arc<dyn PaymentRunRepository>,
        open_items_repo: Arc<dyn OpenItemRepository>,
    ) -> Self {
        Self {
            payment_proposal_repo,
            payment_run_repo,
            open_items_repo,
        }
    }
    
    pub async fn create_payment_proposal(&self, proposal: PaymentProposal) -> RepositoryResult<String> {
        self.payment_proposal_repo.save(&proposal).await?;
        Ok(proposal.proposal_id)
    }
    
    pub async fn approve_payment_proposal(&self, proposal_id: Uuid, user_id: Uuid) -> RepositoryResult<()> {
        self.payment_proposal_repo.update_status(proposal_id, PaymentProposalStatus::Approved, user_id).await
    }
    
    pub async fn execute_payment_run(&self, run: PaymentRun) -> RepositoryResult<String> {
        self.payment_run_repo.save(&run).await?;
        Ok(run.run_id)
    }
    
    pub async fn update_payment_run_status(&self, run_id: Uuid, status: PaymentRunStatus) -> RepositoryResult<()> {
        self.payment_run_repo.update_status(run_id, status).await
    }
}
