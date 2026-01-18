//! GL Service gRPC Client
//!
//! Shared GL Client for all FI services (AP, AR, CO, TR) to create journal entries.
//! This eliminates code duplication and ensures consistent GL integration across services.

use tonic::transport::Channel;
use prost_types::Timestamp;
use chrono::{NaiveDate, Datelike};
use std::sync::Arc;
use tokio::sync::Mutex;

// Import generated GL proto types
pub mod proto {
    pub mod fi {
        pub mod gl {
            pub mod v1 {
                tonic::include_proto!("fi.gl.v1");
            }
        }
    }
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("common.v1");
        }
    }
}

use proto::fi::gl::v1 as gl_v1;
use gl_v1::gl_journal_entry_service_client::GlJournalEntryServiceClient;

/// GL Service Client wrapper
pub struct GlClient {
    client: GlJournalEntryServiceClient<Channel>,
}

impl GlClient {
    /// Create a new GL client connected to the given endpoint.
    ///
    /// # Arguments
    /// * `endpoint` - GL service endpoint (e.g., "http://gl-service:50060")
    ///
    /// # Example
    /// ```no_run
    /// let client = GlClient::new("http://localhost:50060").await?;
    /// ```
    pub async fn new(endpoint: &str) -> Result<Self, tonic::transport::Error> {
        let client = GlJournalEntryServiceClient::connect(endpoint.to_string()).await?;
        Ok(Self { client })
    }

    /// Create a journal entry in GL for financial transactions.
    ///
    /// # Arguments
    /// * `company_code` - Company code
    /// * `document_date` - Document date
    /// * `posting_date` - Posting date
    /// * `fiscal_year` - Fiscal year
    /// * `currency` - Currency code (e.g., "USD")
    /// * `reference_document` - Optional reference document number
    /// * `header_text` - Optional header text description
    /// * `line_items` - Journal entry line items (debits and credits)
    ///
    /// # Returns
    /// GL journal entry response with document number
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
            document_type: "KR".to_string(), // Vendor Invoice (can be parameterized if needed)
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

        let proto_line_items: Vec<gl_v1::JournalEntryLineItem> = line_items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                gl_v1::JournalEntryLineItem {
                    line_item_number: (i + 1) as i32,
                    posting_key: if item.debit_credit == "S" {
                        "40".to_string() // Debit
                    } else {
                        "50".to_string() // Credit
                    },
                    debit_credit_indicator: item.debit_credit.clone(),
                    gl_account: item.gl_account,
                    account_type: 0, // General
                    business_partner: item.business_partner.unwrap_or_default(),
                    amount_in_local_currency: Some(proto::common::v1::MonetaryValue {
                        value: item.amount.to_string(),
                        currency_code: currency.to_string(),
                    }),
                    amount_in_document_currency: Some(proto::common::v1::MonetaryValue {
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
            })
            .collect();

        let request = tonic::Request::new(gl_v1::CreateJournalEntryRequest {
            header: Some(header),
            line_items: proto_line_items,
            test_run: false,
            post_immediately: true, // Auto-post the journal entry
            context: None,
        });

        let response = self.client.create_journal_entry(request).await?;
        Ok(response.into_inner())
    }
}

/// Simplified GL Line Item for AP/AR/CO/TR integration
#[derive(Debug, Clone)]
pub struct GlLineItem {
    pub gl_account: String,
    pub debit_credit: String, // "S" (Soll/Debit) or "H" (Haben/Credit)
    pub amount: rust_decimal::Decimal,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub item_text: Option<String>,
    pub business_partner: Option<String>,
}

/// Convert NaiveDate to protobuf Timestamp
fn naive_date_to_timestamp(date: NaiveDate) -> Timestamp {
    let datetime = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    Timestamp {
        seconds: datetime.timestamp(),
        nanos: 0,
    }
}

/// Create a new GL client with unified initialization.
///
/// This function provides a standardized way to initialize GL clients across all FI services.
/// It reads the GL service endpoint from the environment variable `GL_SERVICE_URL`,
/// or uses the provided default endpoint.
///
/// # Arguments
/// * `default_endpoint` - Default GL service endpoint if `GL_SERVICE_URL` is not set
///
/// # Returns
/// Arc<Mutex<GlClient>> for safe concurrent access
///
/// # Example
/// ```no_run
/// let gl_client = cuba_finance::gl_client::create_gl_client(
///     "http://gl-service.cuba-fi.svc.cluster.local:50060"
/// ).await?;
/// ```
pub async fn create_gl_client(
    default_endpoint: &str,
) -> Result<Arc<Mutex<GlClient>>, Box<dyn std::error::Error>> {
    let endpoint = std::env::var("GL_SERVICE_URL").unwrap_or_else(|_| default_endpoint.to_string());
    tracing::info!("GL Service endpoint: {}", endpoint);

    let client = GlClient::new(&endpoint)
        .await
        .map_err(|e| format!("Failed to connect to GL service: {}", e))?;

    Ok(Arc::new(Mutex::new(client)))
}
