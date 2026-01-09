//! AR/AP Service - Repository Traits
//!
//! 仓储接口定义

use async_trait::async_trait;
use uuid::Uuid;

use super::entities::*;

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

#[async_trait]
pub trait BusinessPartnerRepository: Send + Sync {
    async fn find_by_id(&self, partner_id: &str) -> RepositoryResult<Option<BusinessPartner>>;
    async fn save(&self, partner: &BusinessPartner) -> RepositoryResult<()>;
}

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn find_by_id(&self, customer_id: &str) -> RepositoryResult<Option<Customer>>;
    async fn find_by_company(&self, company_code: &str) -> RepositoryResult<Vec<Customer>>;
    async fn save(&self, customer: &Customer) -> RepositoryResult<()>;
}

#[async_trait]
pub trait SupplierRepository: Send + Sync {
    async fn find_by_id(&self, supplier_id: &str) -> RepositoryResult<Option<Supplier>>;
    async fn find_by_company(&self, company_code: &str) -> RepositoryResult<Vec<Supplier>>;
    async fn save(&self, supplier: &Supplier) -> RepositoryResult<()>;
}

#[async_trait]
pub trait OpenItemRepository: Send + Sync {
    async fn find_by_partner(&self, partner_id: &str, company_code: &str) -> RepositoryResult<Vec<OpenItem>>;
    async fn find_open_items(&self, partner_id: &str) -> RepositoryResult<Vec<OpenItem>>;
    async fn get_balance(&self, partner_id: &str, company_code: &str) -> RepositoryResult<AccountBalance>;
    async fn save(&self, item: &OpenItem) -> RepositoryResult<()>;
}
