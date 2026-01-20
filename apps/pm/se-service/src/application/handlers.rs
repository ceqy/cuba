use crate::application::commands::{AwardCommand, CreateRFQCommand, SubmitQuoteCommand};
use crate::domain::{QuoteItem, RFQ, RFQItem, SupplierQuote};
use crate::infrastructure::repository::SourcingRepository;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct SourcingHandler {
    repo: Arc<SourcingRepository>,
}

impl SourcingHandler {
    pub fn new(repo: Arc<SourcingRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_rfq(&self, cmd: CreateRFQCommand) -> Result<String> {
        let rfq_id = Uuid::new_v4();
        let rfq_number = format!("RFQ{}", Utc::now().timestamp_subsec_micros());

        let rfq = RFQ {
            rfq_id,
            rfq_number: rfq_number.clone(),
            company_code: cmd.company_code,
            purchasing_org: cmd.purchasing_org,
            quote_deadline: Some((Utc::now() + chrono::Duration::days(30)).date_naive()),
            status: "DRAFT".to_string(),
            created_at: Utc::now(),
            items: vec![RFQItem {
                item_id: Uuid::new_v4(),
                rfq_id,
                item_number: 10,
                material: "DEFAULT-MATERIAL".to_string(),
                description: Some("RFQ Item".to_string()),
                quantity: Some(rust_decimal::Decimal::new(100, 0)),
                unit: "EA".to_string(),
                delivery_date: Some((Utc::now() + chrono::Duration::days(60)).date_naive()),
            }],
        };

        self.repo.save_rfq(&rfq).await?;
        Ok(rfq_number)
    }

    pub async fn submit_quote(&self, cmd: SubmitQuoteCommand) -> Result<String> {
        let rfq = self
            .repo
            .find_rfq_by_number(&cmd.rfq_number)
            .await?
            .ok_or_else(|| anyhow::anyhow!("RFQ not found"))?;

        let quote_id = Uuid::new_v4();
        let quote_number = format!("QT{}", Utc::now().timestamp_subsec_micros());

        let quote = SupplierQuote {
            quote_id,
            quote_number: quote_number.clone(),
            rfq_id: rfq.rfq_id,
            supplier_id: cmd.supplier_id,
            validity_end_date: Some((Utc::now() + chrono::Duration::days(90)).date_naive()),
            status: "SUBMITTED".to_string(),
            created_at: Utc::now(),
            items: rfq
                .items
                .into_iter()
                .map(|item| QuoteItem {
                    quote_item_id: Uuid::new_v4(),
                    quote_id,
                    rfq_item_number: item.item_number,
                    quantity: item.quantity,
                    unit: item.unit,
                    net_price: Some(rust_decimal::Decimal::new(50, 0)),
                    currency: "CNY".to_string(),
                    notes: None,
                })
                .collect(),
        };

        self.repo.save_quote(&quote).await?;
        Ok(quote_number)
    }

    pub async fn award_quote(&self, _cmd: AwardCommand) -> Result<bool> {
        Ok(true)
    }
}
