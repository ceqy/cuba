use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct LineItemDTO {
    pub account_id: String,
    pub debit_credit: String, // "D" or "C"
    pub amount: Decimal,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub text: Option<String>,
    pub special_gl_indicator: Option<String>, // UMSKZ: A, F, V, W, or empty
    pub ledger: Option<String>,               // RLDNR: 0L, 1L, 2L...
    pub ledger_type: Option<i32>,             // 1=Leading, 2=NonLeading, 3=Extension
    pub ledger_amount: Option<Decimal>,       // Amount in ledger currency
    pub financial_area: Option<String>,       // RFAREA: Financial area for consolidation
    pub business_area: Option<String>,        // RBUSA: Business area for segment reporting
    pub controlling_area: Option<String>,     // KOKRS: Controlling area for management accounting
    pub account_assignment: Option<String>,   // KTOSL: 科目分配（自动科目确定）
    pub payment_execution: Option<PaymentExecutionDTO>, // 付款执行详细信息
    pub payment_terms_detail: Option<PaymentTermsDetailDTO>, // 付款条件详细信息
    pub business_partner: Option<String>,     // KUNNR/LIFNR: 业务伙伴编号
    pub business_partner_type: Option<String>, // 业务伙伴类型: CUSTOMER/VENDOR
    pub maturity_date: Option<NaiveDate>,     // FAEDT: 到期日
    pub invoice_reference: Option<InvoiceReferenceDTO>, // 发票参考
    pub dunning_detail: Option<DunningDetailDTO>, // 催款详细信息
    pub transaction_type: Option<String>,     // VRGNG 业务交易类型
    pub reference_transaction_type: Option<String>, // AWTYP 参考交易类型
    pub trading_partner_company: Option<String>, // VBUND 交易伙伴公司
    pub amount_in_object_currency: Option<Decimal>, // OSL 对象货币金额
    pub object_currency: Option<String>,      // 对象货币代码
    pub amount_in_profit_center_currency: Option<Decimal>, // VSL 利润中心货币金额
    pub profit_center_currency: Option<String>, // 利润中心货币代码
    pub amount_in_group_currency: Option<Decimal>, // 集团货币金额
    pub group_currency: Option<String>,       // 集团货币代码
}

/// 发票参考 DTO
#[derive(Debug, Deserialize)]
pub struct InvoiceReferenceDTO {
    pub reference_document_number: Option<String>,
    pub reference_fiscal_year: Option<i32>,
    pub reference_line_item: Option<i32>,
    pub reference_document_type: Option<String>,
    pub reference_company_code: Option<String>,
}

/// 催款详细信息 DTO
#[derive(Debug, Deserialize)]
pub struct DunningDetailDTO {
    pub dunning_key: Option<String>,
    pub dunning_block: Option<String>,
    pub last_dunning_date: Option<NaiveDate>,
    pub dunning_date: Option<NaiveDate>,
    pub dunning_level: i32,
    pub dunning_area: Option<String>,
    pub grace_period_days: i32,
    pub dunning_charges: Option<Decimal>,
    pub dunning_clerk: Option<String>,
}

/// 付款执行 DTO
#[derive(Debug, Deserialize)]
pub struct PaymentExecutionDTO {
    pub payment_method: String,                   // ZLSCH 付款方式
    pub house_bank: Option<String>,               // HBKID 内部银行账户
    pub partner_bank_type: Option<String>,        // BVTYP 业务伙伴银行类型
    pub payment_block: Option<String>,            // ZLSPR 付款冻结
    pub payment_baseline_date: Option<NaiveDate>, // ZFBDT 付款基准日
    pub payment_reference: Option<String>,        // 付款参考号
    pub payment_priority: Option<i32>,            // 付款优先级
}

/// 付款条件详细信息 DTO
/// 注意: 使用 i32 而非 Option<i32>,因为 0 值有业务含义 (无折扣/立即)
#[derive(Debug, Deserialize)]
pub struct PaymentTermsDetailDTO {
    pub baseline_date: Option<NaiveDate>,    // ZFBDT 现金折扣基准日
    pub discount_days_1: i32,                // ZBD1T 第一个折扣天数 (0=无折扣)
    pub discount_days_2: i32,                // ZBD2T 第二个折扣天数 (0=无折扣)
    pub net_payment_days: i32,               // ZBD3T 净付款天数 (0=立即)
    pub discount_percent_1: Option<Decimal>, // ZBD1P 第一个折扣百分比
    pub discount_percent_2: Option<Decimal>, // ZBD2P 第二个折扣百分比
    pub discount_amount: Option<Decimal>,    // SKFBT 现金折扣金额
}

#[derive(Debug, Deserialize)]
pub struct CreateJournalEntryCommand {
    pub company_code: String,
    pub fiscal_year: i32,
    pub posting_date: NaiveDate,
    pub document_date: NaiveDate,
    pub currency: String,
    pub reference: Option<String>,
    pub lines: Vec<LineItemDTO>,
    pub post_immediately: bool,
}

#[derive(Debug, Deserialize)]
pub struct PostJournalEntryCommand {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ReverseJournalEntryCommand {
    pub id: Uuid,
    pub reversal_reason: String,
    pub posting_date: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct ParkJournalEntryCommand {
    pub id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJournalEntryCommand {
    pub id: Uuid,
    pub posting_date: Option<NaiveDate>,
    pub document_date: Option<NaiveDate>,
    pub reference: Option<String>,
    pub lines: Option<Vec<LineItemDTO>>,
}
