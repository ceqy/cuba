//! AR/AP Service - PostgreSQL Repository Implementation

use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::*;

pub struct PgArApRepository {
    pool: Arc<PgPool>,
}

impl PgArApRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl BusinessPartnerRepository for PgArApRepository {
    async fn find_by_id(&self, partner_id: &str) -> RepositoryResult<Option<BusinessPartner>> {
        let row = sqlx::query_as!(
            BusinessPartnerRow,
            r#"
            SELECT id, partner_id, partner_type as "partner_type: _", 
                   name_org1, name_last, name_first, search_term, country,
                   created_at, updated_at
            FROM business_partners
            WHERE partner_id = $1
            "#,
            partner_id
        )
        .fetch_optional(&*self.pool)
        .await?;
        
        Ok(row.map(|r| r.into()))
    }
    
    async fn save(&self, partner: &BusinessPartner) -> RepositoryResult<()> {
        let partner_type_str = match partner.partner_type {
            PartnerType::Person => "PERSON",
            PartnerType::Organization => "ORGANIZATION",
        };
        
        sqlx::query!(
            r#"
            INSERT INTO business_partners (id, partner_id, partner_type, name_org1, name_last, name_first, search_term, country)
            VALUES ($1, $2, $3::partner_type, $4, $5, $6, $7, $8)
            ON CONFLICT (partner_id) DO UPDATE SET
                name_org1 = EXCLUDED.name_org1,
                name_last = EXCLUDED.name_last,
                name_first = EXCLUDED.name_first,
                search_term = EXCLUDED.search_term,
                country = EXCLUDED.country,
                updated_at = NOW()
            "#,
            partner.id,
            partner.partner_id,
            partner_type_str,
            partner.name_org1,
            partner.name_last,
            partner.name_first,
            partner.search_term,
            partner.country
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
}

#[async_trait]
impl CustomerRepository for PgArApRepository {
    async fn find_by_id(&self, customer_id: &str) -> RepositoryResult<Option<Customer>> {
        let row = sqlx::query_as!(
            CustomerRow,
            r#"
            SELECT id, customer_id, partner_id, company_code, reconciliation_account,
                   payment_terms, credit_limit, credit_currency, created_at
            FROM customers
            WHERE customer_id = $1
            "#,
            customer_id
        )
        .fetch_optional(&*self.pool)
        .await?;
        
        Ok(row.map(|r| r.into()))
    }
    
    async fn find_by_company(&self, company_code: &str) -> RepositoryResult<Vec<Customer>> {
        let rows = sqlx::query_as!(
            CustomerRow,
            r#"
            SELECT id, customer_id, partner_id, company_code, reconciliation_account,
                   payment_terms, credit_limit, credit_currency, created_at
            FROM customers
            WHERE company_code = $1
            "#,
            company_code
        )
        .fetch_all(&*self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    async fn save(&self, customer: &Customer) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO customers (id, customer_id, partner_id, company_code, reconciliation_account, payment_terms, credit_limit, credit_currency)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (customer_id) DO UPDATE SET
                reconciliation_account = EXCLUDED.reconciliation_account,
                payment_terms = EXCLUDED.payment_terms,
                credit_limit = EXCLUDED.credit_limit,
                credit_currency = EXCLUDED.credit_currency
            "#,
            customer.id,
            customer.customer_id,
            customer.partner_id,
            customer.company_code,
            customer.reconciliation_account,
            customer.payment_terms,
            customer.credit_limit,
            customer.credit_currency
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
}

#[async_trait]
impl SupplierRepository for PgArApRepository {
    async fn find_by_id(&self, supplier_id: &str) -> RepositoryResult<Option<Supplier>> {
        let row = sqlx::query_as!(
            SupplierRow,
            r#"
            SELECT id, supplier_id, partner_id, company_code, reconciliation_account,
                   payment_terms, created_at
            FROM suppliers
            WHERE supplier_id = $1
            "#,
            supplier_id
        )
        .fetch_optional(&*self.pool)
        .await?;
        
        Ok(row.map(|r| r.into()))
    }
    
    async fn find_by_company(&self, company_code: &str) -> RepositoryResult<Vec<Supplier>> {
        let rows = sqlx::query_as!(
            SupplierRow,
            r#"
            SELECT id, supplier_id, partner_id, company_code, reconciliation_account,
                   payment_terms, created_at
            FROM suppliers
            WHERE company_code = $1
            "#,
            company_code
        )
        .fetch_all(&*self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    async fn save(&self, supplier: &Supplier) -> RepositoryResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO suppliers (id, supplier_id, partner_id, company_code, reconciliation_account, payment_terms)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (supplier_id) DO UPDATE SET
                reconciliation_account = EXCLUDED.reconciliation_account,
                payment_terms = EXCLUDED.payment_terms
            "#,
            supplier.id,
            supplier.supplier_id,
            supplier.partner_id,
            supplier.company_code,
            supplier.reconciliation_account,
            supplier.payment_terms
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
}

#[async_trait]
impl OpenItemRepository for PgArApRepository {
    async fn find_by_partner(&self, partner_id: &str, company_code: &str) -> RepositoryResult<Vec<OpenItem>> {
        let rows = sqlx::query_as!(
            OpenItemRow,
            r#"
            SELECT id, company_code, document_number, fiscal_year, line_item,
                   account_type as "account_type: _", partner_id, posting_date, due_date,
                   amount, currency, open_amount, clearing_date, clearing_doc, created_at
            FROM open_items
            WHERE partner_id = $1 AND company_code = $2
            ORDER BY posting_date DESC
            "#,
            partner_id,
            company_code
        )
        .fetch_all(&*self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    async fn find_open_items(&self, partner_id: &str) -> RepositoryResult<Vec<OpenItem>> {
        let rows = sqlx::query_as!(
            OpenItemRow,
            r#"
            SELECT id, company_code, document_number, fiscal_year, line_item,
                   account_type as "account_type: _", partner_id, posting_date, due_date,
                   amount, currency, open_amount, clearing_date, clearing_doc, created_at
            FROM open_items
            WHERE partner_id = $1 AND clearing_date IS NULL
            ORDER BY due_date ASC
            "#,
            partner_id
        )
        .fetch_all(&*self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    async fn get_balance(&self, partner_id: &str, company_code: &str) -> RepositoryResult<AccountBalance> {
        let result = sqlx::query!(
            r#"
            SELECT 
                COALESCE(SUM(CASE WHEN amount > 0 THEN open_amount ELSE 0 END), 0) as total_debit,
                COALESCE(SUM(CASE WHEN amount < 0 THEN ABS(open_amount) ELSE 0 END), 0) as total_credit,
                COALESCE(SUM(open_amount), 0) as balance,
                COUNT(*) as open_items_count,
                COALESCE(MIN(currency), 'CNY') as currency
            FROM open_items
            WHERE partner_id = $1 AND company_code = $2 AND clearing_date IS NULL
            "#,
            partner_id,
            company_code
        )
        .fetch_one(&*self.pool)
        .await?;
        
        Ok(AccountBalance {
            partner_id: partner_id.to_string(),
            company_code: company_code.to_string(),
            currency: result.currency.unwrap_or_else(|| "CNY".to_string()),
            total_debit: result.total_debit.unwrap_or_default(),
            total_credit: result.total_credit.unwrap_or_default(),
            balance: result.balance.unwrap_or_default(),
            open_items_count: result.open_items_count.unwrap_or(0),
        })
    }
    
    async fn save(&self, item: &OpenItem) -> RepositoryResult<()> {
        let account_type_str = match item.account_type {
            AccountType::Customer => "CUSTOMER",
            AccountType::Supplier => "SUPPLIER",
        };
        
        sqlx::query!(
            r#"
            INSERT INTO open_items (id, company_code, document_number, fiscal_year, line_item,
                                   account_type, partner_id, posting_date, due_date, amount,
                                   currency, open_amount, clearing_date, clearing_doc)
            VALUES ($1, $2, $3, $4, $5, $6::account_type, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (company_code, document_number, fiscal_year, line_item) DO UPDATE SET
                open_amount = EXCLUDED.open_amount,
                clearing_date = EXCLUDED.clearing_date,
                clearing_doc = EXCLUDED.clearing_doc
            "#,
            item.id,
            item.company_code,
            item.document_number,
            item.fiscal_year,
            item.line_item,
            account_type_str,
            item.partner_id,
            item.posting_date,
            item.due_date,
            item.amount,
            item.currency,
            item.open_amount,
            item.clearing_date,
            item.clearing_doc
        )
        .execute(&*self.pool)
        .await?;
        
        Ok(())
    }
}

// ============================================================================
// Row Types for sqlx
// ============================================================================

#[derive(Debug, sqlx::FromRow)]
struct BusinessPartnerRow {
    id: Uuid,
    partner_id: String,
    partner_type: String,
    name_org1: Option<String>,
    name_last: Option<String>,
    name_first: Option<String>,
    search_term: Option<String>,
    country: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<BusinessPartnerRow> for BusinessPartner {
    fn from(row: BusinessPartnerRow) -> Self {
        Self {
            id: row.id,
            partner_id: row.partner_id,
            partner_type: match row.partner_type.as_str() {
                "PERSON" => PartnerType::Person,
                _ => PartnerType::Organization,
            },
            name_org1: row.name_org1,
            name_last: row.name_last,
            name_first: row.name_first,
            search_term: row.search_term,
            country: row.country,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct CustomerRow {
    id: Uuid,
    customer_id: String,
    partner_id: Option<String>,
    company_code: String,
    reconciliation_account: Option<String>,
    payment_terms: Option<String>,
    credit_limit: Option<Decimal>,
    credit_currency: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<CustomerRow> for Customer {
    fn from(row: CustomerRow) -> Self {
        Self {
            id: row.id,
            customer_id: row.customer_id,
            partner_id: row.partner_id.unwrap_or_default(),
            company_code: row.company_code,
            reconciliation_account: row.reconciliation_account,
            payment_terms: row.payment_terms,
            credit_limit: row.credit_limit,
            credit_currency: row.credit_currency,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct SupplierRow {
    id: Uuid,
    supplier_id: String,
    partner_id: Option<String>,
    company_code: String,
    reconciliation_account: Option<String>,
    payment_terms: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<SupplierRow> for Supplier {
    fn from(row: SupplierRow) -> Self {
        Self {
            id: row.id,
            supplier_id: row.supplier_id,
            partner_id: row.partner_id.unwrap_or_default(),
            company_code: row.company_code,
            reconciliation_account: row.reconciliation_account,
            payment_terms: row.payment_terms,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct OpenItemRow {
    id: Uuid,
    company_code: String,
    document_number: String,
    fiscal_year: i32,
    line_item: i32,
    account_type: String,
    partner_id: String,
    posting_date: chrono::NaiveDate,
    due_date: Option<chrono::NaiveDate>,
    amount: Decimal,
    currency: String,
    open_amount: Decimal,
    clearing_date: Option<chrono::NaiveDate>,
    clearing_doc: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<OpenItemRow> for OpenItem {
    fn from(row: OpenItemRow) -> Self {
        Self {
            id: row.id,
            company_code: row.company_code,
            document_number: row.document_number,
            fiscal_year: row.fiscal_year,
            line_item: row.line_item,
            account_type: match row.account_type.as_str() {
                "CUSTOMER" => AccountType::Customer,
                _ => AccountType::Supplier,
            },
            partner_id: row.partner_id,
            posting_date: row.posting_date,
            due_date: row.due_date,
            amount: row.amount,
            currency: row.currency,
            open_amount: row.open_amount,
            clearing_date: row.clearing_date,
            clearing_doc: row.clearing_doc,
            created_at: row.created_at,
        }
    }
}
