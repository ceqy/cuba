//! AR/AP Service - Extended Repository Traits
//!
//! Repository interfaces for Phase 2 entities

use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

use super::entities::*;
use super::extended_entities::*;
use super::value_objects::*;

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Entity not found: {0}")]
    NotFound(String),
    
    #[error("Duplicate key: {0}")]
    DuplicateKey(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

// ============================================================================
// Clearing Repository
// ============================================================================

#[async_trait]
pub trait ClearingRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<ClearingHistory>>;
    async fn find_by_document(&self, company_code: &str, clearing_document: &str, fiscal_year: i32) 
        -> RepositoryResult<Option<ClearingHistory>>;
    async fn save(&self, clearing: &ClearingHistory) -> RepositoryResult<()>;
    async fn list_by_company(&self, company_code: &str, from_date: Option<NaiveDate>, to_date: Option<NaiveDate>) 
        -> RepositoryResult<Vec<ClearingHistory>>;
}

// ============================================================================
// Credit Check Repository
// ============================================================================

#[async_trait]
pub trait CreditCheckRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<CreditCheck>>;
    async fn find_latest_by_customer(&self, customer_id: &str) -> RepositoryResult<Option<CreditCheck>>;
    async fn list_by_customer(&self, customer_id: &str, limit: i32) -> RepositoryResult<Vec<CreditCheck>>;
    async fn save(&self, check: &CreditCheck) -> RepositoryResult<()>;
}

// ============================================================================
// Payment Proposal Repository
// ============================================================================

#[async_trait]
pub trait PaymentProposalRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<PaymentProposal>>;
    async fn find_by_proposal_id(&self, proposal_id: &str) -> RepositoryResult<Option<PaymentProposal>>;
    async fn list_by_status(&self, company_code: &str, status: PaymentProposalStatus) 
        -> RepositoryResult<Vec<PaymentProposal>>;
    async fn save(&self, proposal: &PaymentProposal) -> RepositoryResult<()>;
    async fn update_status(&self, id: Uuid, status: PaymentProposalStatus, user_id: Uuid) 
        -> RepositoryResult<()>;
}

// ============================================================================
// Dunning Repository
// ============================================================================

#[async_trait]
pub trait DunningRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<DunningHistory>>;
    async fn list_by_customer(&self, customer_id: &str) -> RepositoryResult<Vec<DunningHistory>>;
    async fn save(&self, dunning: &DunningHistory) -> RepositoryResult<()>;
}

// ============================================================================
// Bank Account Repository
// ============================================================================

#[async_trait]
pub trait BankAccountRepository: Send + Sync {
    async fn find_by_id(&self, account_id: &str) -> RepositoryResult<Option<BankAccount>>;
    async fn list_by_partner(&self, partner_id: &str) -> RepositoryResult<Vec<BankAccount>>;
    async fn find_primary(&self, partner_id: &str) -> RepositoryResult<Option<BankAccount>>;
    async fn save(&self, account: &BankAccount) -> RepositoryResult<()>;
}

// ============================================================================
// Payment Run Repository
// ============================================================================

#[async_trait]
pub trait PaymentRunRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<PaymentRun>>;
    async fn find_by_run_id(&self, run_id: &str) -> RepositoryResult<Option<PaymentRun>>;
    async fn list_by_company(&self, company_code: &str, from_date: Option<NaiveDate>, to_date: Option<NaiveDate>) 
        -> RepositoryResult<Vec<PaymentRun>>;
    async fn save(&self, run: &PaymentRun) -> RepositoryResult<()>;
    async fn update_status(&self, id: Uuid, status: PaymentRunStatus) -> RepositoryResult<()>;
}

// ============================================================================
// Advance Payment Repository
// ============================================================================

#[async_trait]
pub trait AdvancePaymentRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> RepositoryResult<Option<AdvancePayment>>;
    async fn find_by_advance_id(&self, advance_id: &str) -> RepositoryResult<Option<AdvancePayment>>;
    async fn list_active_by_partner(&self, partner_id: &str, company_code: &str) 
        -> RepositoryResult<Vec<AdvancePayment>>;
    async fn save(&self, advance: &AdvancePayment) -> RepositoryResult<()>;
}
