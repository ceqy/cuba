//! PostgreSQL Repository implementations for AP Service

use sqlx::{PgPool, Row};
use uuid::Uuid;
use crate::domain::{Supplier, Invoice, OpenItem, InvoiceItem};
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

/// Invoice Repository
pub struct InvoiceRepository {
    pool: PgPool,
}

impl InvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Invoice>, sqlx::Error> {
        // Fetch header
        let invoice = sqlx::query_as::<_, Invoice>(
            r#"
            SELECT id, document_number, company_code, fiscal_year, document_type,
                   supplier_id, document_date, posting_date, due_date, baseline_date,
                   currency, total_amount, tax_amount, reference_document, header_text,
                   status, clearing_document, clearing_date, created_at, updated_at
            FROM invoices WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(mut inv) = invoice {
            // Fetch items
            let items = sqlx::query_as::<_, InvoiceItem>(
                r#"
                SELECT id, invoice_id, line_item_number, gl_account, debit_credit_indicator,
                       amount, cost_center, profit_center, item_text, purchase_order,
                       po_item_number, goods_receipt, gr_item_number, quantity, unit_of_measure
                FROM invoice_items WHERE invoice_id = $1
                ORDER BY line_item_number ASC
                "#
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await?;

            inv.items = items;
            Ok(Some(inv))
        } else {
            Ok(None)
        }
    }

    pub async fn save(&self, invoice: &Invoice) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        // Save Header
        sqlx::query(
            r#"
            INSERT INTO invoices (
                id, document_number, company_code, fiscal_year, document_type,
                supplier_id, document_date, posting_date, due_date, baseline_date,
                currency, total_amount, tax_amount, reference_document, header_text,
                status, clearing_document, clearing_date, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
            ) 
            ON CONFLICT (document_number, company_code, fiscal_year) DO NOTHING
            "#
        )
        .bind(invoice.id)
        .bind(&invoice.document_number)
        .bind(&invoice.company_code)
        .bind(invoice.fiscal_year)
        .bind(&invoice.document_type)
        .bind(invoice.supplier_id)
        .bind(invoice.document_date)
        .bind(invoice.posting_date)
        .bind(invoice.due_date)
        .bind(invoice.baseline_date)
        .bind(&invoice.currency)
        .bind(invoice.total_amount)
        .bind(invoice.tax_amount)
        .bind(&invoice.reference_document)
        .bind(&invoice.header_text)
        .bind(invoice.status.to_string())
        .bind(&invoice.clearing_document)
        .bind(invoice.clearing_date)
        .bind(invoice.created_at)
        .bind(invoice.updated_at)
        .execute(&mut *tx)
        .await?;

        // Save Items (Blind overwrite or insert? For immutability, just insert)
        for item in &invoice.items {
            sqlx::query(
                r#"
                INSERT INTO invoice_items (
                    id, invoice_id, line_item_number, gl_account, debit_credit_indicator,
                    amount, cost_center, profit_center, item_text, purchase_order,
                    po_item_number, goods_receipt, gr_item_number, quantity, unit_of_measure
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
                )
                ON CONFLICT (invoice_id, line_item_number) DO NOTHING
                "#
            )
            .bind(item.id)
            .bind(item.invoice_id)
            .bind(item.line_item_number)
            .bind(&item.gl_account)
            .bind(item.debit_credit_indicator.as_str())
            .bind(item.amount)
            .bind(&item.cost_center)
            .bind(&item.profit_center)
            .bind(&item.item_text)
            .bind(&item.purchase_order)
            .bind(item.po_item_number)
            .bind(&item.goods_receipt)
            .bind(item.gr_item_number)
            .bind(item.quantity)
            .bind(&item.unit_of_measure)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
