use std::sync::Arc;
use crate::domain::{Invoice, InvoiceItem};
use crate::infrastructure::repository::InvoiceRepository;
use crate::application::commands::{ReceiveInvoiceCommand, MatchInvoiceCommand, PostInvoiceCommand};
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;
use rust_decimal::Decimal;

pub struct InvoiceHandler {
    repo: Arc<InvoiceRepository>,
}

impl InvoiceHandler {
    pub fn new(repo: Arc<InvoiceRepository>) -> Self {
        Self { repo }
    }

    pub async fn receive_invoice(&self, cmd: ReceiveInvoiceCommand) -> Result<String> {
        let invoice_id = Uuid::new_v4();
        let inv = Invoice {
            invoice_id,
            company_code: cmd.company_code,
            supplier_invoice_number: cmd.supplier_invoice_number,
            document_date: cmd.document_date,
            posting_date: None,
            gross_amount: cmd.gross_amount,
            tax_amount: cmd.tax_amount,
            currency: "CNY".to_string(),
            payment_terms: None,
            header_text: None,
            status: "RECEIVED".to_string(),
            document_number: None,
            created_at: Utc::now(),
            items: cmd.items.into_iter().map(|i| InvoiceItem {
                item_id: Uuid::new_v4(),
                invoice_id,
                item_number: i.item_number,
                po_number: i.po_number,
                po_item: i.po_item,
                material: i.material,
                short_text: None,
                quantity: i.quantity,
                unit: "EA".to_string(),
                amount: i.amount,
                tax_code: None,
            }).collect(),
        };
        self.repo.save(&inv).await?;
        Ok(invoice_id.to_string())
    }

    pub async fn match_invoice(&self, cmd: MatchInvoiceCommand) -> Result<(bool, Vec<String>)> {
        // Simplified matching logic - just update status
        self.repo.update_status(cmd.invoice_id, "MATCH_SUCCESS", None).await?;
        Ok((true, vec![]))
    }

    pub async fn post_invoice(&self, cmd: PostInvoiceCommand) -> Result<String> {
        let doc_num = format!("5100{}", Utc::now().timestamp_subsec_micros());
        self.repo.update_status(cmd.invoice_id, "POSTED", Some(&doc_num)).await?;
        Ok(doc_num)
    }
}
