use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::{Customer, OpenItem, Invoice, InvoiceItem};
use anyhow::Result;
use rust_decimal::Decimal;
use uuid::Uuid;

// Define all repositories using the macro
cuba_database::define_repository!(CustomerRepository, OpenItemRepository, InvoiceRepository);

impl CustomerRepository {

    pub async fn save(&self, customer: &Customer) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO customers (
                customer_id, business_partner_id, name, account_group,
                street, city, postal_code, country,
                company_code, reconciliation_account, payment_terms,
                sales_organization, order_currency,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT (customer_id) DO UPDATE SET
                name = EXCLUDED.name,
                updated_at = EXCLUDED.updated_at
            "#)
            .bind(&customer.customer_id)
            .bind(&customer.business_partner_id)
            .bind(&customer.name)
            .bind(&customer.account_group)
            .bind(&customer.street)
            .bind(&customer.city)
            .bind(&customer.postal_code)
            .bind(&customer.country)
            .bind(&customer.company_code)
            .bind(&customer.reconciliation_account)
            .bind(&customer.payment_terms)
            .bind(&customer.sales_organization)
            .bind(&customer.order_currency)
            .bind(&customer.created_at)
            .bind(&customer.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, customer_id: &str) -> Result<Option<Customer>> {
        let rec = sqlx::query_as::<_, Customer>(
            r#"
            SELECT 
                customer_id, business_partner_id, name, account_group,
                street, city, postal_code, country,
                company_code, reconciliation_account, payment_terms,
                sales_organization, distribution_channel, division, order_currency,
                created_at, updated_at
            FROM customers
            WHERE customer_id = $1
            "#)
            .bind(customer_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    pub async fn find_by_customer_id(&self, customer_id: &str) -> Result<Option<Customer>> {
        self.find_by_id(customer_id).await
    }
}

impl OpenItemRepository {

    pub async fn list_by_customer(
        &self,
        customer_id: &str,
        include_cleared: bool,
        limit: i64,
        offset: i64
    ) -> Result<Vec<OpenItem>> {
        let items = sqlx::query_as::<_, OpenItem>(
            r#"
            SELECT 
                open_item_id, document_number, fiscal_year, company_code, line_item_number,
                customer_id, doc_type, posting_date, due_date, currency,
                original_amount, open_amount, is_cleared, payment_block, reference_document, item_text
            FROM open_items
            WHERE customer_id = $1
            AND ($2 = TRUE OR is_cleared = FALSE)
            ORDER BY due_date ASC
            LIMIT $3 OFFSET $4
            "#)
            .bind(customer_id)
            .bind(include_cleared)
            .bind(limit)
            .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }
}

// Invoice Repository (AR Sales Invoices)
impl InvoiceRepository {

    pub async fn save(&self, invoice: &Invoice) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Serialize status to string
        let status_str = match invoice.status {
            crate::domain::InvoiceStatus::Draft => "DRAFT",
            crate::domain::InvoiceStatus::Posted => "POSTED",
            crate::domain::InvoiceStatus::Cancelled => "CANCELLED",
        };

        sqlx::query(
            r#"
            INSERT INTO invoices (
                invoice_id, document_number, company_code, fiscal_year,
                document_date, posting_date, customer_id, currency,
                total_amount, reference, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (invoice_id) DO UPDATE SET
                status = EXCLUDED.status,
                updated_at = EXCLUDED.updated_at
            "#)
            .bind(invoice.invoice_id)
            .bind(&invoice.document_number)
            .bind(&invoice.company_code)
            .bind(invoice.fiscal_year)
            .bind(invoice.document_date)
            .bind(invoice.posting_date)
            .bind(&invoice.customer_id)
            .bind(&invoice.currency)
            .bind(invoice.total_amount)
            .bind(&invoice.reference)
            .bind(status_str)
            .bind(invoice.created_at)
            .bind(invoice.updated_at)
            .execute(&mut *tx)
            .await?;

        // Save items
        for item in &invoice.items {
            sqlx::query(
                r#"
                INSERT INTO invoice_items (
                    item_id, invoice_id, line_item_number, description,
                    quantity, unit_price, total_price, gl_account,
                    tax_code, profit_center
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (item_id) DO NOTHING
                "#)
                .bind(item.item_id)
                .bind(invoice.invoice_id)
                .bind(item.line_item_number)
                .bind(&item.description)
                .bind(item.quantity)
                .bind(item.unit_price)
                .bind(item.total_price)
                .bind(&item.gl_account)
                .bind(&item.tax_code)
                .bind(&item.profit_center)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
