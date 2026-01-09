//! AR/AP Service - Extended PostgreSQL Repository Implementations
//!
//! Repository implementations for Phase 2 entities

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::*;

// Reuse the same pool from PgArApRepository
pub struct PgExtendedRepository {
    pool: Arc<PgPool>,
}

impl PgExtendedRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

// ============================================================================
// Clearing Repository Implementation
// ============================================================================

#[async_trait]
impl ClearingRepository for PgExtendedRepository {
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<ClearingHistory>> {
        // Implementation placeholder - would query clearing_history and join clearing_items
        Ok(None)
    }
    
    async fn find_by_document(&self, company_code: &str, clearing_document: &str, fiscal_year: i32) 
        -> RepositoryResult<Option<ClearingHistory>> {
        // Implementation placeholder
        Ok(None)
    }
    
    async fn save(&self, clearing: &ClearingHistory) -> RepositoryResult<()> {
        let clearing_type_str = clearing.clearing_type.as_str();
        
        sqlx::query!(
            r#"
            INSERT INTO clearing_history (id, clearing_document, company_code, fiscal_year, clearing_date, 
                                         cleared_by, clearing_type, total_amount, currency, reference)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (company_code, clearing_document, fiscal_year) DO UPDATE SET
                total_amount = EXCLUDED.total_amount,
                reference = EXCLUDED.reference
            "#,
            clearing.id,
            clearing.clearing_document,
            clearing.company_code,
            clearing.fiscal_year,
            clearing.clearing_date,
            clearing.cleared_by,
            clearing_type_str,
            clearing.total_amount.amount,
            clearing.total_amount.currency.code(),
            clearing.reference
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn list_by_company(&self, company_code: &str, from_date: Option<NaiveDate>, to_date: Option<NaiveDate>) 
        -> RepositoryResult<Vec<ClearingHistory>> {
        // Implementation placeholder
        Ok(vec![])
    }
}

// ============================================================================
// Credit Check Repository Implementation
// ============================================================================

#[async_trait]
impl CreditCheckRepository for PgExtendedRepository {
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<CreditCheck>> {
        Ok(None)
    }
    
    async fn find_latest_by_customer(&self, customer_id: &str) -> RepositoryResult<Option<CreditCheck>> {
        Ok(None)
    }
    
    async fn list_by_customer(&self, customer_id: &str, limit: i32) -> RepositoryResult<Vec<CreditCheck>> {
        Ok(vec![])
    }
    
    async fn save(&self, check: &CreditCheck) -> RepositoryResult<()> {
        let result_str = check.check_result.as_str();
        
        sqlx::query!(
            r#"
            INSERT INTO credit_checks (id, customer_id, check_date, credit_limit, current_exposure, 
                                      available_credit, check_result, check_reason, checked_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            check.id,
            check.customer_id,
            check.check_date,
            check.credit_limit.as_ref().map(|m| m.amount),
            check.current_exposure.amount,
            check.available_credit.amount,
            result_str,
            check.check_reason,
            check.checked_by
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
}

// ============================================================================
// Payment Proposal Repository Implementation
// ============================================================================

#[async_trait]
impl PaymentProposalRepository for PgExtendedRepository {
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<PaymentProposal>> {
        Ok(None)
    }
    
    async fn find_by_proposal_id(&self, proposal_id: &str) -> RepositoryResult<Option<PaymentProposal>> {
        Ok(None)
    }
    
    async fn list_by_status(&self, company_code: &str, status: PaymentProposalStatus) 
        -> RepositoryResult<Vec<PaymentProposal>> {
        Ok(vec![])
    }
    
    async fn save(&self, proposal: &PaymentProposal) -> RepositoryResult<()> {
        let status_str = proposal.status.as_str();
        let method_str = proposal.payment_method.as_str();
        
        sqlx::query!(
            r#"
            INSERT INTO payment_proposals (id, proposal_id, company_code, proposal_date, payment_date,
                                          payment_method, total_amount, currency, status, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (proposal_id) DO UPDATE SET
                status = EXCLUDED.status
            "#,
            proposal.id,
            proposal.proposal_id,
            proposal.company_code,
            proposal.proposal_date,
            proposal.payment_date,
            method_str,
            proposal.total_amount.amount,
            proposal.total_amount.currency.code(),
            status_str,
            proposal.created_by
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn update_status(&self, id: Uuid, status: PaymentProposalStatus, user_id: Uuid) 
        -> RepositoryResult<()> {
        let status_str = status.as_str();
        
        sqlx::query!(
            r#"
            UPDATE payment_proposals 
            SET status = $1, 
                approved_by = CASE WHEN $1 = 'APPROVED' THEN $2 ELSE approved_by END,
                executed_by = CASE WHEN $1 = 'EXECUTED' THEN $2 ELSE executed_by END
            WHERE id = $3
            "#,
            status_str,
            user_id,
            id
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
}

// ============================================================================
// Dunning Repository Implementation
// ============================================================================

#[async_trait]
impl DunningRepository for PgExtendedRepository {
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<DunningHistory>> {
        Ok(None)
    }
    
    async fn list_by_customer(&self, customer_id: &str) -> RepositoryResult<Vec<DunningHistory>> {
        Ok(vec![])
    }
    
    async fn save(&self, dunning: &DunningHistory) -> RepositoryResult<()> {
        let method_str = dunning.contact_method.as_str();
        
        sqlx::query!(
            r#"
            INSERT INTO dunning_history (id, customer_id, dunning_level, dunning_date, 
                                        total_overdue, currency, contact_method, response, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            dunning.id,
            dunning.customer_id,
            dunning.dunning_level,
            dunning.dunning_date,
            dunning.total_overdue.amount,
            dunning.total_overdue.currency.code(),
            method_str,
            dunning.response,
            dunning.created_by
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
}

// ============================================================================
// Bank Account Repository Implementation
// ============================================================================

#[async_trait]
impl BankAccountRepository for PgExtendedRepository {
    async fn find_by_id(&self, account_id: &str) -> RepositoryResult<Option<BankAccount>> {
        Ok(None)
    }
    
    async fn list_by_partner(&self, partner_id: &str) -> RepositoryResult<Vec<BankAccount>> {
        Ok(vec![])
    }
    
    async fn find_primary(&self, partner_id: &str) -> RepositoryResult<Option<BankAccount>> {
        Ok(None)
    }
    
    async fn save(&self, account: &BankAccount) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO partner_bank_accounts (id, account_id, partner_id, bank_key, account_number,
                                              iban, account_holder, currency, is_primary, is_active)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (account_id) DO UPDATE SET
                is_primary = EXCLUDED.is_primary,
                is_active = EXCLUDED.is_active
            "#,
            account.id,
            account.account_id,
            account.partner_id,
            account.bank_key,
            account.account_number,
            account.iban,
            account.account_holder,
            account.currency.code(),
            account.is_primary,
            account.is_active
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
}

// ============================================================================
// Payment Run Repository Implementation
// ============================================================================

#[async_trait]
impl PaymentRunRepository for PgExtendedRepository {
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<PaymentRun>> {
        Ok(None)
    }
    
    async fn find_by_run_id(&self, run_id: &str) -> RepositoryResult<Option<PaymentRun>> {
        Ok(None)
    }
    
    async fn list_by_company(&self, company_code: &str, from_date: Option<NaiveDate>, to_date: Option<NaiveDate>) 
        -> RepositoryResult<Vec<PaymentRun>> {
        Ok(vec![])
    }
    
    async fn save(&self, run: &PaymentRun) -> RepositoryResult<()> {
        let status_str = run.status.as_str();
        let method_str = run.payment_method.as_str();
        
        sqlx::query!(
            r#"
            INSERT INTO payment_runs (id, run_id, company_code, run_date, payment_method,
                                     total_payments, total_amount, currency, status, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (run_id) DO UPDATE SET
                status = EXCLUDED.status,
                total_payments = EXCLUDED.total_payments,
                total_amount = EXCLUDED.total_amount
            "#,
            run.id,
            run.run_id,
            run.company_code,
            run.run_date,
            method_str,
            run.total_payments,
            run.total_amount.amount,
            run.total_amount.currency.code(),
            status_str,
            run.created_by
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn update_status(&self, id: Uuid, status: PaymentRunStatus) -> RepositoryResult<()> {
        let status_str = status.as_str();
        
        sqlx::query!(
            r#"
            UPDATE payment_runs 
            SET status = $1,
                executed_at = CASE WHEN $1 = 'COMPLETED' THEN NOW() ELSE executed_at END
            WHERE id = $2
            "#,
            status_str,
            id
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
}

// ============================================================================
// Advance Payment Repository Implementation
// ============================================================================

#[async_trait]
impl AdvancePaymentRepository for PgExtendedRepository {
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<AdvancePayment>> {
        Ok(None)
    }
    
    async fn find_by_advance_id(&self, advance_id: &str) -> RepositoryResult<Option<AdvancePayment>> {
        Ok(None)
    }
    
    async fn list_active_by_partner(&self, partner_id: &str, company_code: &str) 
        -> RepositoryResult<Vec<AdvancePayment>> {
        Ok(vec![])
    }
    
    async fn save(&self, advance: &AdvancePayment) -> RepositoryResult<()> {
        let account_type_str = match advance.account_type {
            AccountType::Customer => "CUSTOMER",
            AccountType::Supplier => "SUPPLIER",
        };
        let status_str = advance.status.as_str();
        
        sqlx::query!(
            r#"
            INSERT INTO advance_payments (id, advance_id, company_code, partner_id, account_type,
                                         posting_date, amount, currency, remaining_amount, 
                                         gl_account, reference, status)
            VALUES ($1, $2, $3, $4, $5::account_type, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (advance_id) DO UPDATE SET
                remaining_amount = EXCLUDED.remaining_amount,
                status = EXCLUDED.status
            "#,
            advance.id,
            advance.advance_id,
            advance.company_code,
            advance.partner_id,
            account_type_str,
            advance.posting_date,
            advance.amount.amount,
            advance.amount.currency.code(),
            advance.remaining_amount.amount,
            advance.gl_account,
            advance.reference,
            status_str
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
}
