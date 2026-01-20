use crate::domain::{QuoteItem, RFQ, RFQItem, SupplierQuote};
use anyhow::Result;
use sqlx::PgPool;

pub struct SourcingRepository {
    pool: PgPool,
}

impl SourcingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_rfq(&self, rfq: &RFQ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO rfqs (rfq_id, rfq_number, company_code, purchasing_org, quote_deadline, status) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(rfq.rfq_id)
            .bind(&rfq.rfq_number)
            .bind(&rfq.company_code)
            .bind(&rfq.purchasing_org)
            .bind(rfq.quote_deadline)
            .bind(&rfq.status)
        .execute(&mut *tx).await?;

        for item in &rfq.items {
            sqlx::query(
                "INSERT INTO rfq_items (item_id, rfq_id, item_number, material, description, quantity, unit, delivery_date) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
                .bind(item.item_id)
                .bind(item.rfq_id)
                .bind(item.item_number)
                .bind(&item.material)
                .bind(&item.description)
                .bind(item.quantity)
                .bind(&item.unit)
                .bind(item.delivery_date)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_rfq_by_number(&self, rfq_num: &str) -> Result<Option<RFQ>> {
        let h = sqlx::query_as::<_, RFQ>("SELECT rfq_id, rfq_number, company_code, purchasing_org, quote_deadline, status, created_at FROM rfqs WHERE rfq_number = $1")
            .bind(rfq_num)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let items = sqlx::query_as::<_, RFQItem>(
                "SELECT * FROM rfq_items WHERE rfq_id = $1 ORDER BY item_number",
            )
            .bind(h.rfq_id)
            .fetch_all(&self.pool)
            .await?;
            h.items = items;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    pub async fn save_quote(&self, quote: &SupplierQuote) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO supplier_quotes (quote_id, quote_number, rfq_id, supplier_id, validity_end_date, status) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(quote.quote_id)
            .bind(&quote.quote_number)
            .bind(quote.rfq_id)
            .bind(&quote.supplier_id)
            .bind(quote.validity_end_date)
            .bind(&quote.status)
        .execute(&mut *tx).await?;

        for item in &quote.items {
            sqlx::query(
                "INSERT INTO quote_items (quote_item_id, quote_id, rfq_item_number, quantity, unit, net_price, currency, notes) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
                .bind(item.quote_item_id)
                .bind(item.quote_id)
                .bind(item.rfq_item_number)
                .bind(item.quantity)
                .bind(&item.unit)
                .bind(item.net_price)
                .bind(&item.currency)
                .bind(&item.notes)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_quote_by_number(&self, quote_num: &str) -> Result<Option<SupplierQuote>> {
        let h = sqlx::query_as::<_, SupplierQuote>("SELECT quote_id, quote_number, rfq_id, supplier_id, validity_end_date, status, created_at FROM supplier_quotes WHERE quote_number = $1")
            .bind(quote_num)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let items =
                sqlx::query_as::<_, QuoteItem>("SELECT * FROM quote_items WHERE quote_id = $1")
                    .bind(h.quote_id)
                    .fetch_all(&self.pool)
                    .await?;
            h.items = items;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }
}
