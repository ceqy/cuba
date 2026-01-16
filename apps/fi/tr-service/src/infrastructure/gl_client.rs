//! GL Service gRPC Client
//! Used to create journal entries in General Ledger when invoices are posted.

use tonic::transport::Channel;
use prost_types::Timestamp;
use chrono::{NaiveDate, Datelike};

// Import GL proto types (generated from build.rs)
use crate::api::proto::fi::gl::v1 as gl_v1;
use gl_v1::gl_journal_entry_service_client::GlJournalEntryServiceClient;

/// GL Service Client wrapper
pub struct GlClient {
    client: GlJournalEntryServiceClient<Channel>,
}

impl GlClient {
    /// Create a new GL client connected to the given endpoint.
    /// Default: `http://localhost:50051` in local dev, `http://gl-service:50051` in K8s.
    pub async fn new(endpoint: &str) -> Result<Self, tonic::transport::Error> {
        let client = GlJournalEntryServiceClient::connect(endpoint.to_string()).await?;
        Ok(Self { client })
    }

    /// Create a journal entry in GL for an AP invoice posting.
    /// Debit: Expense account (from invoice items)
    /// Credit: Vendor payable account (reconciliation_account)
    pub async fn create_invoice_journal_entry(
        &mut self,
        company_code: &str,
        document_date: NaiveDate,
        posting_date: NaiveDate,
        fiscal_year: i32,
        currency: &str,
        reference_document: Option<String>,
        header_text: Option<String>,
        line_items: Vec<GlLineItem>,
    ) -> Result<gl_v1::JournalEntryResponse, tonic::Status> {
        let header = gl_v1::JournalEntryHeader {
            company_code: company_code.to_string(),
            document_type: "KR".to_string(), // Vendor Invoice
            document_date: Some(naive_date_to_timestamp(document_date)),
            posting_date: Some(naive_date_to_timestamp(posting_date)),
            fiscal_year,
            fiscal_period: posting_date.month() as i32,
            currency: currency.to_string(),
            exchange_rate: "1.0".to_string(),
            reference_document: reference_document.unwrap_or_default(),
            header_text: header_text.unwrap_or_default(),
            origin: gl_v1::DocumentOrigin::Api as i32,
            logical_system: "".to_string(),
            ledger_group: "".to_string(),
            audit: None,
        };

        let proto_line_items: Vec<gl_v1::JournalEntryLineItem> = line_items.into_iter().enumerate().map(|(i, item)| {
            gl_v1::JournalEntryLineItem {
                line_item_number: (i + 1) as i32,
                posting_key: if item.debit_credit == "S" { "40".to_string() } else { "50".to_string() }, // 40=Debit, 50=Credit
                debit_credit_indicator: item.debit_credit.clone(),
                gl_account: item.gl_account,
                account_type: 0, // General
                business_partner: item.business_partner.unwrap_or_default(),
                amount_in_local_currency: Some(crate::api::proto::common::v1::MonetaryValue {
                    value: item.amount.to_string(),
                    currency_code: currency.to_string(),
                }),
                amount_in_document_currency: Some(crate::api::proto::common::v1::MonetaryValue {
                    value: item.amount.to_string(),
                    currency_code: currency.to_string(),
                }),
                amount_in_group_currency: None,
                cost_center: item.cost_center.unwrap_or_default(),
                profit_center: item.profit_center.unwrap_or_default(),
                segment: "".to_string(),
                internal_order: "".to_string(),
                wbs_element: "".to_string(),
                text: item.item_text.unwrap_or_default(),
                assignment_number: "".to_string(),
                tax_code: "".to_string(),
                tax_jurisdiction: "".to_string(),
                clearing_document: "".to_string(),
                clearing_date: None,
                quantity: None,
            }
        }).collect();

        let request = tonic::Request::new(gl_v1::CreateJournalEntryRequest {
            header: Some(header),
            line_items: proto_line_items,
            test_run: false,
            post_immediately: true, // Auto-post the journal entry
            context: None,
        });

        let response: tonic::Response<gl_v1::JournalEntryResponse> = self.client.create_journal_entry(request).await?;
        Ok(response.into_inner())
    }
}

/// Simplified GL Line Item for AP/AR integration
pub struct GlLineItem {
    pub gl_account: String,
    pub debit_credit: String, // "S" or "H"
    pub amount: rust_decimal::Decimal,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub item_text: Option<String>,
    pub business_partner: Option<String>,
}

fn naive_date_to_timestamp(date: NaiveDate) -> Timestamp {
    let datetime = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    Timestamp {
        seconds: datetime.timestamp(),
        nanos: 0,
    }
}
