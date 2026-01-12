//! PostgreSQL Repository implementations for AP Service

use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::domain::{Supplier, Invoice, InvoiceStatus, OpenItem, DebitCredit, InvoiceItem};
use chrono::{DateTime, Utc};

/// Supplier Repository
pub struct SupplierRepository {
    pool: PgPool,
}

impl SupplierRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Supplier>, sqlx::Error> {
        let row = sqlx::query_as::<_, Supplier>(
            r#"
            SELECT id, supplier_id, business_partner_id, name, account_group,
                   street, city, postal_code, country, telephone, email,
                   company_code, reconciliation_account, payment_terms, check_double_invoice,
                   purchasing_organization, order_currency, created_at, updated_at
            FROM suppliers WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn find_by_supplier_id(&self, supplier_id: &str) -> Result<Option<Supplier>, sqlx::Error> {
        let row = sqlx::query_as::<_, Supplier>(
            r#"
            SELECT id, supplier_id, business_partner_id, name, account_group,
                   street, city, postal_code, country, telephone, email,
                   company_code, reconciliation_account, payment_terms, check_double_invoice,
                   purchasing_organization, order_currency, created_at, updated_at
            FROM suppliers WHERE supplier_id = $1
            "#
        )
        .bind(supplier_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row)
    }

    pub async fn save(&self, supplier: &Supplier) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO suppliers (
                id, supplier_id, business_partner_id, name, account_group,
                street, city, postal_code, country, telephone, email,
                company_code, reconciliation_account, payment_terms, check_double_invoice,
                purchasing_organization, order_currency, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19
            )
            ON CONFLICT (supplier_id) DO UPDATE SET
                business_partner_id = EXCLUDED.business_partner_id,
                name = EXCLUDED.name,
                account_group = EXCLUDED.account_group,
                street = EXCLUDED.street,
                city = EXCLUDED.city,
                postal_code = EXCLUDED.postal_code,
                country = EXCLUDED.country,
                telephone = EXCLUDED.telephone,
                email = EXCLUDED.email,
                reconciliation_account = EXCLUDED.reconciliation_account,
                payment_terms = EXCLUDED.payment_terms,
                check_double_invoice = EXCLUDED.check_double_invoice,
                purchasing_organization = EXCLUDED.purchasing_organization,
                order_currency = EXCLUDED.order_currency,
                updated_at = NOW()
            "#
        )
        .bind(supplier.id)
        .bind(&supplier.supplier_id)
        .bind(&supplier.business_partner_id)
        .bind(&supplier.name)
        .bind(&supplier.account_group)
        .bind(&supplier.street)
        .bind(&supplier.city)
        .bind(&supplier.postal_code)
        .bind(&supplier.country)
        .bind(&supplier.telephone)
        .bind(&supplier.email)
        .bind(&supplier.company_code)
        .bind(&supplier.reconciliation_account)
        .bind(&supplier.payment_terms)
        .bind(supplier.check_double_invoice)
        .bind(&supplier.purchasing_organization)
        .bind(&supplier.order_currency)
        .bind(supplier.created_at)
        .bind(supplier.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Open Items Repository
pub struct OpenItemRepository {
    pool: PgPool,
}

impl OpenItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_supplier(
        &self, 
        supplier_id: Uuid, 
        company_code: &str,
        include_cleared: bool
    ) -> Result<Vec<OpenItem>, sqlx::Error> {
        let query = if include_cleared {
            r#"
            SELECT id, document_number, company_code, fiscal_year, line_item_number,
                   supplier_id, account_type, posting_date, due_date, baseline_date,
                   currency, original_amount, open_amount, is_cleared, clearing_document,
                   clearing_date, reference_document, item_text, payment_block, created_at, updated_at
            FROM open_items
            WHERE supplier_id = $1 AND company_code = $2
            ORDER BY due_date ASC
            "#
        } else {
            r#"
            SELECT id, document_number, company_code, fiscal_year, line_item_number,
                   supplier_id, account_type, posting_date, due_date, baseline_date,
                   currency, original_amount, open_amount, is_cleared, clearing_document,
                   clearing_date, reference_document, item_text, payment_block, created_at, updated_at
            FROM open_items
            WHERE supplier_id = $1 AND company_code = $2 AND is_cleared = false
            ORDER BY due_date ASC
            "#
        };

        let items = sqlx::query_as::<_, OpenItem>(query)
            .bind(supplier_id)
            .bind(company_code)
            .fetch_all(&self.pool)
            .await?;

        Ok(items)
    }

    pub async fn save(&self, item: &OpenItem) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO open_items (
                id, document_number, company_code, fiscal_year, line_item_number,
                supplier_id, account_type, posting_date, due_date, baseline_date,
                currency, original_amount, open_amount, is_cleared, clearing_document,
                clearing_date, reference_document, item_text, payment_block, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21
            )
            ON CONFLICT (document_number, company_code, fiscal_year, line_item_number) DO UPDATE SET
                open_amount = EXCLUDED.open_amount,
                is_cleared = EXCLUDED.is_cleared,
                clearing_document = EXCLUDED.clearing_document,
                clearing_date = EXCLUDED.clearing_date,
                updated_at = NOW()
            "#
        )
        .bind(item.id)
        .bind(&item.document_number)
        .bind(&item.company_code)
        .bind(item.fiscal_year)
        .bind(item.line_item_number)
        .bind(item.supplier_id)
        .bind(&item.account_type)
        .bind(item.posting_date)
        .bind(item.due_date)
        .bind(item.baseline_date)
        .bind(&item.currency)
        .bind(item.original_amount)
        .bind(item.open_amount)
        .bind(item.is_cleared)
        .bind(&item.clearing_document)
        .bind(item.clearing_date)
        .bind(&item.reference_document)
        .bind(&item.item_text)
        .bind(&item.payment_block)
        .bind(item.created_at)
        .bind(item.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
