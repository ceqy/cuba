use crate::domain::{Invoice, InvoiceItem};
use anyhow::Result;
use sqlx::PgPool;

pub struct InvoiceRepository {
    pool: PgPool,
}

impl InvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, inv: &Invoice) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO invoices (invoice_id, company_code, supplier_invoice_number, document_date, gross_amount, tax_amount, currency, payment_terms, header_text, status) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
            .bind(inv.invoice_id)
            .bind(&inv.company_code)
            .bind(&inv.supplier_invoice_number)
            .bind(inv.document_date)
            .bind(inv.gross_amount)
            .bind(inv.tax_amount)
            .bind(&inv.currency)
            .bind(&inv.payment_terms)
            .bind(&inv.header_text)
            .bind(&inv.status)
        .execute(&mut *tx).await?;

        for item in &inv.items {
            sqlx::query(
                "INSERT INTO invoice_items (item_id, invoice_id, item_number, po_number, po_item, material, short_text, quantity, unit, amount, tax_code) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)")
                .bind(item.item_id)
                .bind(item.invoice_id)
                .bind(item.item_number)
                .bind(&item.po_number)
                .bind(item.po_item)
                .bind(&item.material)
                .bind(&item.short_text)
                .bind(item.quantity)
                .bind(&item.unit)
                .bind(item.amount)
                .bind(&item.tax_code)
            .execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_by_id(&self, invoice_id: uuid::Uuid) -> Result<Option<Invoice>> {
        let h = sqlx::query_as::<_, Invoice>("SELECT invoice_id, company_code, supplier_invoice_number, document_date, posting_date, gross_amount, tax_amount, currency, payment_terms, header_text, status, document_number, created_at FROM invoices WHERE invoice_id = $1")
            .bind(invoice_id)
            .fetch_optional(&self.pool).await?;
        if let Some(mut h) = h {
            let items = sqlx::query_as::<_, InvoiceItem>(
                "SELECT * FROM invoice_items WHERE invoice_id = $1 ORDER BY item_number",
            )
            .bind(invoice_id)
            .fetch_all(&self.pool)
            .await?;
            h.items = items;
            Ok(Some(h))
        } else {
            Ok(None)
        }
    }

    pub async fn update_status(
        &self,
        invoice_id: uuid::Uuid,
        status: &str,
        doc_num: Option<&str>,
    ) -> Result<()> {
        sqlx::query("UPDATE invoices SET status = $1, document_number = $2, posting_date = CURRENT_DATE WHERE invoice_id = $3")
            .bind(status)
            .bind(doc_num)
            .bind(invoice_id)
            .execute(&self.pool).await?;
        Ok(())
    }
}
