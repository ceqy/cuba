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
