use sqlx::PgPool;
use crate::domain::{Invoice, InvoiceItem};
use anyhow::Result;

pub struct InvoiceRepository {
    pool: PgPool,
}

impl InvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, inv: &Invoice) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query!(
            "INSERT INTO invoices (invoice_id, company_code, supplier_invoice_number, document_date, gross_amount, tax_amount, currency, payment_terms, header_text, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            inv.invoice_id, inv.company_code, inv.supplier_invoice_number, inv.document_date, inv.gross_amount, inv.tax_amount, inv.currency, inv.payment_terms, inv.header_text, inv.status
        ).execute(&mut *tx).await?;

        for item in &inv.items {
            sqlx::query!(
                "INSERT INTO invoice_items (item_id, invoice_id, item_number, po_number, po_item, material, short_text, quantity, unit, amount, tax_code) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                item.item_id, item.invoice_id, item.item_number, item.po_number, item.po_item, item.material, item.short_text, item.quantity, item.unit, item.amount, item.tax_code
            ).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_id(&self, invoice_id: uuid::Uuid) -> Result<Option<Invoice>> {
        let h = sqlx::query!("SELECT * FROM invoices WHERE invoice_id = $1", invoice_id)
            .fetch_optional(&self.pool).await?;
        if let Some(h) = h {
            let items = sqlx::query!("SELECT * FROM invoice_items WHERE invoice_id = $1 ORDER BY item_number", invoice_id)
                .fetch_all(&self.pool).await?;
            Ok(Some(Invoice {
                invoice_id: h.invoice_id,
                company_code: h.company_code,
                supplier_invoice_number: h.supplier_invoice_number,
                document_date: h.document_date,
                posting_date: h.posting_date,
                gross_amount: h.gross_amount,
                tax_amount: h.tax_amount.unwrap_or_default(),
                currency: h.currency.unwrap_or_else(|| "CNY".to_string()),
                payment_terms: h.payment_terms,
                header_text: h.header_text,
                status: h.status.unwrap_or_else(|| "RECEIVED".to_string()),
                document_number: h.document_number,
                created_at: h.created_at,
                items: items.into_iter().map(|i| InvoiceItem {
                    item_id: i.item_id,
                    invoice_id: i.invoice_id,
                    item_number: i.item_number,
                    po_number: i.po_number,
                    po_item: i.po_item,
                    material: i.material,
                    short_text: i.short_text,
                    quantity: i.quantity,
                    unit: i.unit.unwrap_or_else(|| "EA".to_string()),
                    amount: i.amount,
                    tax_code: i.tax_code,
                }).collect(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_status(&self, invoice_id: uuid::Uuid, status: &str, doc_num: Option<&str>) -> Result<()> {
        sqlx::query!("UPDATE invoices SET status = $1, document_number = $2, posting_date = CURRENT_DATE WHERE invoice_id = $3",
            status, doc_num, invoice_id
        ).execute(&self.pool).await?;
        Ok(())
    }
}
