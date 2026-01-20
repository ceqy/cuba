//! GL Service gRPC Client
//!
//! Shared GL Client for all FI services (AP, AR, CO, TR) to create journal entries.
//! This eliminates code duplication and ensures consistent GL integration across services.

use chrono::{Datelike, NaiveDate};
use prost_types::Timestamp;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;

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

use gl_v1::gl_journal_entry_service_client::GlJournalEntryServiceClient;
use proto::fi::gl::v1 as gl_v1;

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

    /// 在总账中创建会计分录
    ///
    /// # 参数
    /// * `company_code` - 公司代码
    /// * `document_date` - 凭证日期
    /// * `posting_date` - 过账日期
    /// * `fiscal_year` - 会计年度
    /// * `currency` - 货币代码 (例如 "USD")
    /// * `reference_document` - 可选的参考凭证号
    /// * `header_text` - 可选的抬头文本描述
    /// * `line_items` - 会计分录行项目（借方和贷方）
    /// * `ledger_id` - 可选的分类账 ID（默认为 "0L" 主分类账）
    ///
    /// # 返回
    /// 包含凭证编号的总账分录响应
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
        ledger_id: Option<&str>,
    ) -> Result<gl_v1::JournalEntryResponse, tonic::Status> {
        // 使用提供的 ledger_id 或默认使用 "0L"（主分类账）
        let default_ledger = ledger_id.unwrap_or("0L").to_string();

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
            default_ledger: default_ledger.clone(),
            audit: None,
            local_currency: currency.to_string(),
            group_currency: "".to_string(),
            target_currency: "".to_string(),
            chart_of_accounts: "".to_string(),
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
                    special_gl_indicator: item.special_gl_indicator.unwrap_or_default(),
                    // 使用行项目指定的分类账，如果没有则使用默认分类账
                    ledger: item.ledger.unwrap_or_else(|| default_ledger.clone()),
                    ledger_type: item.ledger_type.unwrap_or(1), // 默认为 Leading (1)
                    amount_in_ledger_currency: Some(proto::common::v1::MonetaryValue {
                        value: item.amount.to_string(),
                        currency_code: currency.to_string(),
                    }),
                    // 组织维度字段
                    financial_area: item.financial_area.unwrap_or_default(),
                    business_area: item.business_area.unwrap_or_default(),
                    controlling_area: item.controlling_area.unwrap_or_default(),
                    // 新增字段
                    account_assignment: "".to_string(),
                    amount_in_object_currency: None,
                    amount_in_profit_center_currency: None,
                    transaction_type: "".to_string(),
                    reference_transaction_type: "".to_string(),
                    trading_partner_company: "".to_string(),
                    // 其他字段
                    payment_terms_detail: None,
                    payment_execution: None,
                    invoice_reference: None,
                    dunning_detail: None,
                    internal_trading_detail: None,
                    field_split_detail: None,
                    local_gaap_detail: None,
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
    pub special_gl_indicator: Option<String>, // UMSKZ: A, F, V, W
    pub ledger: Option<String>,               // 分类账
    pub ledger_type: Option<i32>,             // 分类账类型
    pub financial_area: Option<String>,       // RFAREA: 财务范围
    pub business_area: Option<String>,        // RBUSA: 业务范围
    pub controlling_area: Option<String>,     // KOKRS: 控制范围
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
