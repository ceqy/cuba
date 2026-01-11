use sqlx::PgPool;
use crate::domain::{RFQ, RFQItem, SupplierQuote, QuoteItem};
use anyhow::Result;

pub struct SourcingRepository {
    pool: PgPool,
}

impl SourcingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_rfq(&self, rfq: &RFQ) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "INSERT INTO rfqs (rfq_id, rfq_number, company_code, purchasing_org, quote_deadline, status) VALUES ($1, $2, $3, $4, $5, $6)",
            rfq.rfq_id, rfq.rfq_number, rfq.company_code, rfq.purchasing_org, rfq.quote_deadline, rfq.status
        ).execute(&mut *tx).await?;

        for item in &rfq.items {
            sqlx::query!(
                "INSERT INTO rfq_items (item_id, rfq_id, item_number, material, description, quantity, unit, delivery_date) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                item.item_id, item.rfq_id, item.item_number, item.material, item.description, item.quantity, item.unit, item.delivery_date
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_rfq_by_number(&self, rfq_num: &str) -> Result<Option<RFQ>> {
        let h = sqlx::query!("SELECT * FROM rfqs WHERE rfq_number = $1", rfq_num)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let items = sqlx::query!("SELECT * FROM rfq_items WHERE rfq_id = $1 ORDER BY item_number", h.rfq_id)
                .fetch_all(&self.pool).await?;
            Ok(Some(RFQ {
                rfq_id: h.rfq_id,
                rfq_number: h.rfq_number,
                company_code: h.company_code,
                purchasing_org: h.purchasing_org,
                quote_deadline: h.quote_deadline,
                status: h.status.unwrap_or_default(),
                created_at: h.created_at,
                items: items.into_iter().map(|i| RFQItem {
                    item_id: i.item_id,
                    rfq_id: i.rfq_id,
                    item_number: i.item_number,
                    material: i.material,
                    description: i.description,
                    quantity: i.quantity,
                    unit: i.unit.unwrap_or_default(),
                    delivery_date: i.delivery_date,
                }).collect(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_quote(&self, quote: &SupplierQuote) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "INSERT INTO supplier_quotes (quote_id, quote_number, rfq_id, supplier_id, validity_end_date, status) VALUES ($1, $2, $3, $4, $5, $6)",
            quote.quote_id, quote.quote_number, quote.rfq_id, quote.supplier_id, quote.validity_end_date, quote.status
        ).execute(&mut *tx).await?;

        for item in &quote.items {
            sqlx::query!(
                "INSERT INTO quote_items (quote_item_id, quote_id, rfq_item_number, quantity, unit, net_price, currency, notes) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                item.quote_item_id, item.quote_id, item.rfq_item_number, item.quantity, item.unit, item.net_price, item.currency, item.notes
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_quote_by_number(&self, quote_num: &str) -> Result<Option<SupplierQuote>> {
        let h = sqlx::query!("SELECT * FROM supplier_quotes WHERE quote_number = $1", quote_num)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let items = sqlx::query!("SELECT * FROM quote_items WHERE quote_id = $1", h.quote_id)
                .fetch_all(&self.pool).await?;
            Ok(Some(SupplierQuote {
                quote_id: h.quote_id,
                quote_number: h.quote_number,
                rfq_id: h.rfq_id,
                supplier_id: h.supplier_id,
                validity_end_date: h.validity_end_date,
                status: h.status.unwrap_or_default(),
                created_at: h.created_at,
                items: items.into_iter().map(|i| QuoteItem {
                    quote_item_id: i.quote_item_id,
                    quote_id: i.quote_id,
                    rfq_item_number: i.rfq_item_number,
                    quantity: i.quantity,
                    unit: i.unit.unwrap_or_default(),
                    net_price: i.net_price,
                    currency: i.currency.unwrap_or_default(),
                    notes: i.notes,
                }).collect(),
            }))
        } else {
            Ok(None)
        }
    }
}
