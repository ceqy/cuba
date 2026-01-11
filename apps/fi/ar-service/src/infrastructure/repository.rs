use async_trait::async_trait;
use sqlx::PgPool;
use crate::domain::{Customer, OpenItem};
use anyhow::Result;
use rust_decimal::Decimal;

pub struct CustomerRepository {
    pool: PgPool,
}

impl CustomerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, customer: &Customer) -> Result<()> {
        sqlx::query!(
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
            "#,
            customer.customer_id,
            customer.business_partner_id,
            customer.name,
            customer.account_group,
            customer.street,
            customer.city,
            customer.postal_code,
            customer.country,
            customer.company_code,
            customer.reconciliation_account,
            customer.payment_terms,
            customer.sales_organization,
            customer.order_currency,
            customer.created_at,
            customer.updated_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, customer_id: &str) -> Result<Option<Customer>> {
        let rec = sqlx::query_as!(
            Customer,
            r#"
            SELECT 
                customer_id, business_partner_id, name, account_group,
                street, city, postal_code, country,
                company_code, reconciliation_account, payment_terms,
                sales_organization, distribution_channel, division, order_currency,
                created_at, updated_at
            FROM customers
            WHERE customer_id = $1
            "#,
            customer_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }
}

pub struct OpenItemRepository {
    pool: PgPool,
}

impl OpenItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_customer(
        &self,
        customer_id: &str,
        include_cleared: bool,
        limit: i64,
        offset: i64
    ) -> Result<Vec<OpenItem>> {
        let items = sqlx::query!(
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
            "#,
            customer_id,
            include_cleared,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let result = items.into_iter().map(|rec| OpenItem {
            open_item_id: rec.open_item_id,
            document_number: rec.document_number,
            fiscal_year: rec.fiscal_year,
            company_code: rec.company_code,
            line_item_number: rec.line_item_number,
            customer_id: rec.customer_id,
            doc_type: rec.doc_type,
            posting_date: rec.posting_date,
            due_date: rec.due_date,
            currency: rec.currency,
            original_amount: rec.original_amount,
            open_amount: rec.open_amount,
            is_cleared: rec.is_cleared,
            payment_block: rec.payment_block,
            reference_document: rec.reference_document,
            item_text: rec.item_text,
        }).collect();

        Ok(result)
    }
}
