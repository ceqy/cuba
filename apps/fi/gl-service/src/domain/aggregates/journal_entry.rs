use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{NaiveDate, DateTime, Utc};
use uuid::Uuid;
use cuba_core::domain::AggregateRoot;
use thiserror::Error;

/// 付款执行详细信息 (Payment Execution Detail)
/// 用于自动付款程序（Automatic Payment Program）和付款执行
/// SAP 字段映射: ZLSCH, HBKID, BVTYP, ZLSPR
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaymentExecutionDetail {
    pub payment_method: String,           // ZLSCH 付款方式（T-转账、C-支票、W-电汇、Z-其他）
    pub house_bank: Option<String>,       // HBKID 内部银行账户标识（公司银行账户）
    pub partner_bank_type: Option<String>, // BVTYP 业务伙伴银行类型
    pub payment_block: Option<String>,    // ZLSPR 付款冻结（冻结原因代码）
    pub payment_baseline_date: Option<NaiveDate>, // ZFBDT 付款基准日
    pub payment_reference: Option<String>, // 付款参考号
    pub payment_priority: Option<i32>,    // 付款优先级（1-9，数字越小优先级越高）
}

/// 付款条件详细信息 (Payment Terms Detail)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentTermsDetail {
    pub baseline_date: Option<NaiveDate>,    // ZFBDT 现金折扣基准日
    pub discount_days_1: i32,                // ZBD1T 第一个折扣天数
    pub discount_days_2: i32,                // ZBD2T 第二个折扣天数
    pub net_payment_days: i32,               // ZBD3T 净付款天数
    pub discount_percent_1: Option<Decimal>, // ZBD1P 折扣百分比
    pub discount_percent_2: Option<Decimal>, // ZBD2P 折扣百分比
    pub discount_amount: Option<Decimal>,    // SKFBT 现金折扣金额
}

/// 发票参考信息 (Invoice Reference)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvoiceReference {
    pub reference_document_number: Option<String>,
    pub reference_fiscal_year: Option<i32>,
    pub reference_line_item: Option<i32>,
    pub reference_document_type: Option<String>,
    pub reference_company_code: Option<String>,
}

/// 催款详细信息 (Dunning Detail)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DunningDetail {
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

impl PaymentExecutionDetail {
    /// 创建新的付款执行详细信息
    pub fn new(payment_method: String) -> Self {
        Self {
            payment_method,
            house_bank: None,
            partner_bank_type: None,
            payment_block: None,
            payment_baseline_date: None,
            payment_reference: None,
            payment_priority: None,
        }
    }

    /// 创建完整的付款执行详细信息
    pub fn with_details(
        payment_method: String,
        house_bank: Option<String>,
        partner_bank_type: Option<String>,
    ) -> Self {
        Self {
            payment_method,
            house_bank,
            partner_bank_type,
            payment_block: None,
            payment_baseline_date: None,
            payment_reference: None,
            payment_priority: None,
        }
    }

    /// 判断付款是否被冻结
    pub fn is_blocked(&self) -> bool {
        self.payment_block.is_some()
    }

    /// 设置付款冻结
    pub fn with_payment_block(mut self, block_reason: String) -> Self {
        self.payment_block = Some(block_reason);
        self
    }

    /// 设置付款基准日
    pub fn with_baseline_date(mut self, date: NaiveDate) -> Self {
        self.payment_baseline_date = Some(date);
        self
    }

    /// 设置付款参考号
    pub fn with_reference(mut self, reference: String) -> Self {
        self.payment_reference = Some(reference);
        self
    }

    /// 设置付款优先级
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.payment_priority = Some(priority);
        self
    }

    /// 获取付款方式描述
    pub fn payment_method_description(&self) -> &str {
        match self.payment_method.as_str() {
            "T" => "银行转账 (Bank Transfer)",
            "C" => "支票 (Check)",
            "W" => "电汇 (Wire Transfer)",
            "Z" => "其他 (Other)",
            _ => "未知付款方式",
        }
    }

    /// 验证付款执行信息
    pub fn validate(&self) -> Result<(), String> {
        if self.payment_method.is_empty() {
            return Err("付款方式不能为空".to_string());
        }

        if let Some(priority) = self.payment_priority {
            if !(1..=9).contains(&priority) {
                return Err("付款优先级必须在 1-9 之间".to_string());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostingStatus {
    Draft,
    Parked,
    Posted,
    Reversed,
}

impl Default for PostingStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl  ToString for PostingStatus {
    fn to_string(&self) -> String {
        match self {
            PostingStatus::Draft => "DRAFT".to_string(),
            PostingStatus::Parked => "PARKED".to_string(),
            PostingStatus::Posted => "POSTED".to_string(),
            PostingStatus::Reversed => "REVERSED".to_string(),
        }
    }
}

impl std::str::FromStr for PostingStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "DRAFT" => Ok(PostingStatus::Draft),
            "PARKED" => Ok(PostingStatus::Parked),
            "POSTED" => Ok(PostingStatus::Posted),
            "REVERSED" => Ok(PostingStatus::Reversed),
            _ => Err(format!("Invalid status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebitCredit {
    Debit,
    Credit,
}

impl DebitCredit {
    pub fn as_char(&self) -> char {
        match self {
            DebitCredit::Debit => 'D',
            DebitCredit::Credit => 'C',
        }
    }

    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'D' => Some(DebitCredit::Debit),
            'C' => Some(DebitCredit::Credit),
            _ => None,
        }
    }
}

/// 分类账类型（并行会计）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LedgerType {
    Leading = 1,      // 主分类账 (0L)
    NonLeading = 2,   // 非主分类账 (1L, 2L)
    Extension = 3,    // 扩展分类账
}

impl Default for LedgerType {
    fn default() -> Self {
        Self::Leading
    }
}

impl From<i32> for LedgerType {
    fn from(value: i32) -> Self {
        match value {
            1 => LedgerType::Leading,
            2 => LedgerType::NonLeading,
            3 => LedgerType::Extension,
            _ => LedgerType::Leading,
        }
    }
}

impl From<LedgerType> for i32 {
    fn from(value: LedgerType) -> Self {
        value as i32
    }
}

/// 特殊总账类型 (Special G/L Type / UMSKZ)
/// 用于区分特殊业务类型，影响应收/应付账款分类、报表列示和清账逻辑
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialGlType {
    Normal,           // 普通业务（无特殊标识）
    BillOfExchange,   // A - 票据 (Bills of Exchange)
    DownPayment,      // F - 预付款 (Down Payment)
    AdvancePayment,   // V - 预收款 (Advance Payment)
    BillDiscount,     // W - 票据贴现 (Bill of Exchange Discount)
}

impl Default for SpecialGlType {
    fn default() -> Self {
        Self::Normal
    }
}

impl SpecialGlType {
    /// 转换为 SAP UMSKZ 字符
    pub fn to_sap_code(&self) -> &str {
        match self {
            SpecialGlType::Normal => "",
            SpecialGlType::BillOfExchange => "A",
            SpecialGlType::DownPayment => "F",
            SpecialGlType::AdvancePayment => "V",
            SpecialGlType::BillDiscount => "W",
        }
    }

    /// 从 SAP UMSKZ 字符转换
    pub fn from_sap_code(code: &str) -> Self {
        match code {
            "A" => SpecialGlType::BillOfExchange,
            "F" => SpecialGlType::DownPayment,
            "V" => SpecialGlType::AdvancePayment,
            "W" => SpecialGlType::BillDiscount,
            _ => SpecialGlType::Normal,
        }
    }

    /// 获取描述
    pub fn description(&self) -> &str {
        match self {
            SpecialGlType::Normal => "普通业务",
            SpecialGlType::BillOfExchange => "票据 (Bills of Exchange)",
            SpecialGlType::DownPayment => "预付款 (Down Payment)",
            SpecialGlType::AdvancePayment => "预收款 (Advance Payment)",
            SpecialGlType::BillDiscount => "票据贴现 (Bill of Exchange Discount)",
        }
    }

    /// 判断是否为特殊总账业务
    pub fn is_special(&self) -> bool {
        !matches!(self, SpecialGlType::Normal)
    }

    /// 判断是否为预付款
    pub fn is_down_payment(&self) -> bool {
        matches!(self, SpecialGlType::DownPayment)
    }

    /// 判断是否为预收款
    pub fn is_advance_payment(&self) -> bool {
        matches!(self, SpecialGlType::AdvancePayment)
    }

    /// 判断是否为票据相关业务
    pub fn is_bill_related(&self) -> bool {
        matches!(self, SpecialGlType::BillOfExchange | SpecialGlType::BillDiscount)
    }

    /// 获取英文名称
    pub fn english_name(&self) -> &str {
        match self {
            SpecialGlType::Normal => "Normal",
            SpecialGlType::BillOfExchange => "Bill of Exchange",
            SpecialGlType::DownPayment => "Down Payment",
            SpecialGlType::AdvancePayment => "Advance Payment",
            SpecialGlType::BillDiscount => "Bill Discount",
        }
    }

    /// 获取所有有效的特殊总账类型
    pub fn all_special_types() -> Vec<SpecialGlType> {
        vec![
            SpecialGlType::BillOfExchange,
            SpecialGlType::DownPayment,
            SpecialGlType::AdvancePayment,
            SpecialGlType::BillDiscount,
        ]
    }

    /// 获取所有类型（包括普通业务）
    pub fn all_types() -> Vec<SpecialGlType> {
        vec![
            SpecialGlType::Normal,
            SpecialGlType::BillOfExchange,
            SpecialGlType::DownPayment,
            SpecialGlType::AdvancePayment,
            SpecialGlType::BillDiscount,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    pub id: Uuid,
    pub line_number: i32,
    pub account_id: String,
    pub debit_credit: DebitCredit,
    pub amount: Decimal,
    pub local_amount: Decimal,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub text: Option<String>,

    // ============================================================================
    // 特殊总账标识 (Special G/L Indicator / UMSKZ)
    // ============================================================================
    pub special_gl_indicator: SpecialGlType,   // 特殊总账类型

    // ============================================================================
    // 并行会计字段 (Parallel Accounting)
    // ============================================================================
    pub ledger: String,                    // 分类账编号 (RLDNR: 0L, 1L, 2L...)
    pub ledger_type: LedgerType,           // 分类账类型
    pub ledger_amount: Option<Decimal>,    // 分类账货币金额（用于不同会计准则）

    // ============================================================================
    // 组织维度字段 (Organizational Dimensions)
    // ============================================================================
    pub financial_area: Option<String>,    // 财务范围 (RFAREA) - 用于合并报表
    pub business_area: Option<String>,     // 业务范围 (RBUSA) - 用于段报告
    pub controlling_area: Option<String>,  // 控制范围 (KOKRS) - 用于管理会计

    // ============================================================================
    // 科目分配字段 (Account Assignment / KTOSL)
    // ============================================================================
    pub account_assignment: Option<String>, // 科目分配 (KTOSL) - 用于自动科目确定

    // ============================================================================
    // 业务伙伴字段
    // ============================================================================
    pub business_partner: Option<String>,     // 业务伙伴编号 (KUNNR/LIFNR)
    pub business_partner_type: Option<String>, // 业务伙伴类型

    // ============================================================================
    // 付款执行字段 (Payment Execution / ZLSCH)
    // ============================================================================
    pub payment_execution: Option<PaymentExecutionDetail>, // 付款执行详细信息
    pub payment_terms_detail: Option<PaymentTermsDetail>,  // 付款条件详细信息
    pub invoice_reference: Option<InvoiceReference>,       // 发票参考
    pub dunning_detail: Option<DunningDetail>,             // 催款详细信息
    pub transaction_type: Option<String>,                  // 业务交易类型
    pub reference_transaction_type: Option<String>,        // 参考交易类型
    pub trading_partner_company: Option<String>,           // 交易伙伴公司
    pub amount_in_object_currency: Option<Decimal>,        // 对象货币金额
    pub object_currency: Option<String>,                   // 对象货币代码
    pub amount_in_profit_center_currency: Option<Decimal>, // 利润中心货币金额
    pub profit_center_currency: Option<String>,            // 利润中心货币代码
    pub amount_in_group_currency: Option<Decimal>,         // 集团货币金额
    pub group_currency: Option<String>,                    // 集团货币代码
    pub maturity_date: Option<NaiveDate>,                  // 到期日
}

impl LineItem {
    /// 创建默认使用主分类账 (0L) 的行项目
    pub fn new(
        line_number: i32,
        account_id: String,
        debit_credit: DebitCredit,
        amount: Decimal,
        local_amount: Decimal,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            line_number,
            account_id,
            debit_credit,
            amount,
            local_amount,
            cost_center: None,
            profit_center: None,
            text: None,
            special_gl_indicator: SpecialGlType::Normal,  // 默认普通业务
            ledger: "0L".to_string(),           // 默认主分类账
            ledger_type: LedgerType::Leading,
            ledger_amount: None,
            financial_area: None,
            business_area: None,
            controlling_area: None,
            account_assignment: None,
            business_partner: None,
            business_partner_type: None,
            payment_execution: None,
            payment_terms_detail: None,
            invoice_reference: None,
            dunning_detail: None,
            transaction_type: None,
            reference_transaction_type: None,
            trading_partner_company: None,
            amount_in_object_currency: None,
            object_currency: None,
            amount_in_profit_center_currency: None,
            profit_center_currency: None,
            amount_in_group_currency: None,
            group_currency: None,
            maturity_date: None,
        }
    }

    /// 创建指定分类账的行项目（用于并行会计）
    pub fn with_ledger(
        line_number: i32,
        account_id: String,
        debit_credit: DebitCredit,
        amount: Decimal,
        local_amount: Decimal,
        ledger: String,
        ledger_type: LedgerType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            line_number,
            account_id,
            debit_credit,
            amount,
            local_amount,
            cost_center: None,
            profit_center: None,
            text: None,
            special_gl_indicator: SpecialGlType::Normal,  // 默认普通业务
            ledger,
            ledger_type,
            ledger_amount: Some(amount), // 默认使用相同金额
            financial_area: None,
            business_area: None,
            controlling_area: None,
            account_assignment: None,
            business_partner: None,
            business_partner_type: None,
            payment_execution: None,
            payment_terms_detail: None,
            invoice_reference: None,
            dunning_detail: None,
            transaction_type: None,
            reference_transaction_type: None,
            trading_partner_company: None,
            amount_in_object_currency: None,
            object_currency: None,
            amount_in_profit_center_currency: None,
            profit_center_currency: None,
            amount_in_group_currency: None,
            group_currency: None,
            maturity_date: None,
        }
    }

    /// 创建特殊总账行项目（用于票据、预付款等）
    pub fn with_special_gl(
        line_number: i32,
        account_id: String,
        debit_credit: DebitCredit,
        amount: Decimal,
        local_amount: Decimal,
        special_gl_indicator: SpecialGlType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            line_number,
            account_id,
            debit_credit,
            amount,
            local_amount,
            cost_center: None,
            profit_center: None,
            text: None,
            special_gl_indicator,
            ledger: "0L".to_string(),
            ledger_type: LedgerType::Leading,
            ledger_amount: None,
            financial_area: None,
            business_area: None,
            controlling_area: None,
            account_assignment: None,
            business_partner: None,
            business_partner_type: None,
            payment_execution: None,
            payment_terms_detail: None,
            invoice_reference: None,
            dunning_detail: None,
            transaction_type: None,
            reference_transaction_type: None,
            trading_partner_company: None,
            amount_in_object_currency: None,
            object_currency: None,
            amount_in_profit_center_currency: None,
            profit_center_currency: None,
            amount_in_group_currency: None,
            group_currency: None,
            maturity_date: None,
        }
    }

    /// 判断是否为特殊总账行项目
    pub fn is_special_gl(&self) -> bool {
        self.special_gl_indicator.is_special()
    }

    /// 判断是否为预付款行项目
    pub fn is_down_payment(&self) -> bool {
        self.special_gl_indicator.is_down_payment()
    }

    /// 判断是否为预收款行项目
    pub fn is_advance_payment(&self) -> bool {
        self.special_gl_indicator.is_advance_payment()
    }

    /// 判断是否为票据相关行项目
    pub fn is_bill_related(&self) -> bool {
        self.special_gl_indicator.is_bill_related()
    }

    /// 获取特殊总账类型描述
    pub fn special_gl_description(&self) -> &str {
        self.special_gl_indicator.description()
    }

    /// 设置成本中心
    pub fn with_cost_center(mut self, cost_center: String) -> Self {
        self.cost_center = Some(cost_center);
        self
    }

    /// 设置利润中心
    pub fn with_profit_center(mut self, profit_center: String) -> Self {
        self.profit_center = Some(profit_center);
        self
    }

    /// 设置行项目文本
    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    /// 设置付款执行信息
    pub fn with_payment_execution(mut self, payment_execution: PaymentExecutionDetail) -> Self {
        self.payment_execution = Some(payment_execution);
        self
    }

    /// 创建完整的特殊总账行项目（包含所有可选字段）
    pub fn builder() -> LineItemBuilder {
        LineItemBuilder::new()
    }
}

/// LineItem 构建器，用于创建复杂的行项目
pub struct LineItemBuilder {
    line_number: Option<i32>,
    account_id: Option<String>,
    debit_credit: Option<DebitCredit>,
    amount: Option<Decimal>,
    local_amount: Option<Decimal>,
    cost_center: Option<String>,
    profit_center: Option<String>,
    text: Option<String>,
    special_gl_indicator: SpecialGlType,
    ledger: String,
    ledger_type: LedgerType,
    ledger_amount: Option<Decimal>,
    payment_execution: Option<PaymentExecutionDetail>,
    payment_terms_detail: Option<PaymentTermsDetail>,
    invoice_reference: Option<InvoiceReference>,
    dunning_detail: Option<DunningDetail>,
    transaction_type: Option<String>,
    reference_transaction_type: Option<String>,
    trading_partner_company: Option<String>,
    amount_in_object_currency: Option<Decimal>,
    object_currency: Option<String>,
    amount_in_profit_center_currency: Option<Decimal>,
    profit_center_currency: Option<String>,
    amount_in_group_currency: Option<Decimal>,
    group_currency: Option<String>,
    maturity_date: Option<NaiveDate>,
    business_partner: Option<String>,
    business_partner_type: Option<String>,
}

impl LineItemBuilder {
    pub fn new() -> Self {
        Self {
            line_number: None,
            account_id: None,
            debit_credit: None,
            amount: None,
            local_amount: None,
            cost_center: None,
            profit_center: None,
            text: None,
            special_gl_indicator: SpecialGlType::Normal,
            ledger: "0L".to_string(),
            ledger_type: LedgerType::Leading,
            ledger_amount: None,
            payment_execution: None,
            payment_terms_detail: None,
            invoice_reference: None,
            dunning_detail: None,
            transaction_type: None,
            reference_transaction_type: None,
            trading_partner_company: None,
            amount_in_object_currency: None,
            object_currency: None,
            amount_in_profit_center_currency: None,
            profit_center_currency: None,
            amount_in_group_currency: None,
            group_currency: None,
            maturity_date: None,
            business_partner: None,
            business_partner_type: None,
        }
    }

    pub fn line_number(mut self, line_number: i32) -> Self {
        self.line_number = Some(line_number);
        self
    }

    pub fn account_id(mut self, account_id: String) -> Self {
        self.account_id = Some(account_id);
        self
    }

    pub fn debit_credit(mut self, debit_credit: DebitCredit) -> Self {
        self.debit_credit = Some(debit_credit);
        self
    }

    pub fn amount(mut self, amount: Decimal) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn local_amount(mut self, local_amount: Decimal) -> Self {
        self.local_amount = Some(local_amount);
        self
    }

    pub fn cost_center(mut self, cost_center: String) -> Self {
        self.cost_center = Some(cost_center);
        self
    }

    pub fn profit_center(mut self, profit_center: String) -> Self {
        self.profit_center = Some(profit_center);
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    pub fn special_gl_indicator(mut self, special_gl_indicator: SpecialGlType) -> Self {
        self.special_gl_indicator = special_gl_indicator;
        self
    }

    pub fn ledger(mut self, ledger: String) -> Self {
        self.ledger = ledger;
        self
    }

    pub fn ledger_type(mut self, ledger_type: LedgerType) -> Self {
        self.ledger_type = ledger_type;
        self
    }

    pub fn ledger_amount(mut self, ledger_amount: Decimal) -> Self {
        self.ledger_amount = Some(ledger_amount);
        self
    }

    pub fn payment_execution(mut self, payment_execution: PaymentExecutionDetail) -> Self {
        self.payment_execution = Some(payment_execution);
        self
    }

    pub fn payment_terms_detail(mut self, payment_terms_detail: PaymentTermsDetail) -> Self {
        self.payment_terms_detail = Some(payment_terms_detail);
        self
    }

    pub fn build(self) -> Result<LineItem, String> {
        Ok(LineItem {
            id: Uuid::new_v4(),
            line_number: self.line_number.ok_or("line_number is required")?,
            account_id: self.account_id.ok_or("account_id is required")?,
            debit_credit: self.debit_credit.ok_or("debit_credit is required")?,
            amount: self.amount.ok_or("amount is required")?,
            local_amount: self.local_amount.ok_or("local_amount is required")?,
            cost_center: self.cost_center,
            profit_center: self.profit_center,
            text: self.text,
            special_gl_indicator: self.special_gl_indicator,
            ledger: self.ledger,
            ledger_type: self.ledger_type,
            ledger_amount: self.ledger_amount,
            financial_area: None,
            business_area: None,
            controlling_area: None,
            account_assignment: None,
            business_partner: self.business_partner,
            business_partner_type: self.business_partner_type,
            payment_execution: self.payment_execution,
            payment_terms_detail: self.payment_terms_detail,
            invoice_reference: self.invoice_reference,
            dunning_detail: self.dunning_detail,
            transaction_type: self.transaction_type,
            reference_transaction_type: self.reference_transaction_type,
            trading_partner_company: self.trading_partner_company,
            amount_in_object_currency: self.amount_in_object_currency,
            object_currency: self.object_currency,
            amount_in_profit_center_currency: self.amount_in_profit_center_currency,
            profit_center_currency: self.profit_center_currency,
            amount_in_group_currency: self.amount_in_group_currency,
            group_currency: self.group_currency,
            maturity_date: self.maturity_date,
        })
    }
}

impl Default for LineItemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: Uuid,
    pub document_number: Option<String>,
    pub company_code: String,
    pub fiscal_year: i32,
    pub posting_date: NaiveDate,
    pub document_date: NaiveDate,
    pub currency: String,
    pub reference: Option<String>,
    pub status: PostingStatus,
    pub lines: Vec<LineItem>,
    pub created_at: DateTime<Utc>,
    pub posted_at: Option<DateTime<Utc>>,
    pub tenant_id: Option<String>,

    // ============================================================================
    // 并行会计字段
    // ============================================================================
    pub ledger_group: Option<String>,      // 分类账组 (LDGRP)
    pub default_ledger: String,            // 默认分类账 (RLDNR)

    // ============================================================================
    // 多币种字段 (Multi-Currency Support)
    // ============================================================================
    pub local_currency: String,            // 本位币 (RHCUR)
    pub group_currency: Option<String>,    // 集团货币 (RKCUR)
    pub target_currency: Option<String>,   // 目标货币 (RTCUR)

    // ============================================================================
    // 科目表字段
    // ============================================================================
    pub chart_of_accounts: Option<String>, // 科目表 (KTOPL)
}

#[derive(Error, Debug)]
pub enum JournalEntryError {
    #[error("Debits ({debit}) must equal Credits ({credit})")]
    BalanceError { debit: Decimal, credit: Decimal },
    #[error("Journal entry is already posted")]
    AlreadyPosted,
    #[error("Journal entry is not posted")]
    NotPosted,
    #[error("Empty lines")]
    EmptyLines,
}

impl JournalEntry {
    pub fn new(
        company_code: String,
        fiscal_year: i32,
        posting_date: NaiveDate,
        document_date: NaiveDate,
        currency: String,
        reference: Option<String>,
        lines: Vec<LineItem>,
        tenant_id: Option<String>,
    ) -> Result<Self, JournalEntryError> {
        if lines.is_empty() {
             return Err(JournalEntryError::EmptyLines);
        }

        let entry = Self {
            id: Uuid::new_v4(),
            document_number: None,
            company_code: company_code.clone(),
            fiscal_year,
            posting_date,
            document_date,
            currency: currency.clone(),
            reference,
            status: PostingStatus::Draft,
            lines,
            created_at: Utc::now(),
            posted_at: None,
            tenant_id,
            ledger_group: None,
            default_ledger: "0L".to_string(),
            local_currency: currency.clone(),
            group_currency: None,
            target_currency: None,
            chart_of_accounts: None,
        };

        Ok(entry)
    }

    pub fn validate_balance(&self) -> Result<(), JournalEntryError> {
        let mut debit_sum = Decimal::ZERO;
        let mut credit_sum = Decimal::ZERO;

        for line in &self.lines {
            match line.debit_credit {
                DebitCredit::Debit => debit_sum += line.amount,
                DebitCredit::Credit => credit_sum += line.amount,
            }
        }

        if debit_sum != credit_sum {
            return Err(JournalEntryError::BalanceError { debit: debit_sum, credit: credit_sum });
        }

        Ok(())
    }

    pub fn post(&mut self, document_number: String) -> Result<(), JournalEntryError> {
        if self.status == PostingStatus::Posted {
            return Err(JournalEntryError::AlreadyPosted);
        }

        self.validate_balance()?;

        self.status = PostingStatus::Posted;
        self.document_number = Some(document_number);
        self.posted_at = Some(Utc::now());

        Ok(())
    }

    /// 创建冲销凭证
    pub fn create_reversal_entry(&self, reversal_date: NaiveDate) -> Result<JournalEntry, JournalEntryError> {
        if self.status != PostingStatus::Posted {
            return Err(JournalEntryError::NotPosted);
        }

        // 反转所有行项目的借贷方向
        let reversed_lines: Vec<LineItem> = self.lines.iter().enumerate().map(|(i, line)| LineItem {
            id: Uuid::new_v4(),
            line_number: (i + 1) as i32,
            account_id: line.account_id.clone(),
            debit_credit: match line.debit_credit {
                DebitCredit::Debit => DebitCredit::Credit,
                DebitCredit::Credit => DebitCredit::Debit,
            },
            amount: line.amount,
            local_amount: line.local_amount,
            cost_center: line.cost_center.clone(),
            profit_center: line.profit_center.clone(),
            text: Some(format!("冲销 {}", self.document_number.as_ref().unwrap_or(&"".to_string()))),
            special_gl_indicator: line.special_gl_indicator,  // 保留特殊总账标识
            ledger: line.ledger.clone(),
            ledger_type: line.ledger_type,
            ledger_amount: line.ledger_amount,
            financial_area: line.financial_area.clone(),
            business_area: line.business_area.clone(),
            controlling_area: line.controlling_area.clone(),
            account_assignment: line.account_assignment.clone(),
            business_partner: line.business_partner.clone(),
            business_partner_type: line.business_partner_type.clone(),
            payment_execution: line.payment_execution.clone(),
            payment_terms_detail: line.payment_terms_detail.clone(),
            invoice_reference: line.invoice_reference.clone(),
            dunning_detail: line.dunning_detail.clone(),
            transaction_type: line.transaction_type.clone(),
            reference_transaction_type: line.reference_transaction_type.clone(),
            trading_partner_company: line.trading_partner_company.clone(),
            amount_in_object_currency: line.amount_in_object_currency,
            object_currency: line.object_currency.clone(),
            amount_in_profit_center_currency: line.amount_in_profit_center_currency,
            profit_center_currency: line.profit_center_currency.clone(),
            amount_in_group_currency: line.amount_in_group_currency,
            group_currency: line.group_currency.clone(),
            maturity_date: line.maturity_date,
        }).collect();

        let mut reversal_entry = JournalEntry::new(
            self.company_code.clone(),
            self.fiscal_year,
            reversal_date,
            reversal_date,
            self.currency.clone(),
            Some(format!("冲销 {}", self.document_number.as_ref().unwrap_or(&"".to_string()))),
            reversed_lines,
            self.tenant_id.clone(),
        )?;

        // 自动过账冲销凭证
        let reversal_doc_num = format!("REV-{}", self.document_number.as_ref().unwrap_or(&Uuid::new_v4().simple().to_string()));
        reversal_entry.post(reversal_doc_num)?;

        Ok(reversal_entry)
    }

    /// 标记原凭证为已冲销
    pub fn mark_as_reversed(&mut self) {
        self.status = PostingStatus::Reversed;
    }

    /// 暂存凭证 (Park)
    pub fn park(&mut self) -> Result<(), JournalEntryError> {
        if self.status == PostingStatus::Posted {
            return Err(JournalEntryError::AlreadyPosted);
        }

        // 验证借贷平衡
        self.validate_balance()?;

        self.status = PostingStatus::Parked;
        Ok(())
    }

    /// 更新凭证 (仅限 Draft 或 Parked 状态)
    pub fn update(
        &mut self,
        posting_date: Option<NaiveDate>,
        document_date: Option<NaiveDate>,
        reference: Option<String>,
        lines: Option<Vec<LineItem>>,
    ) -> Result<(), JournalEntryError> {
        if self.status == PostingStatus::Posted || self.status == PostingStatus::Reversed {
            return Err(JournalEntryError::AlreadyPosted);
        }

        if let Some(pd) = posting_date {
            self.posting_date = pd;
        }
        if let Some(dd) = document_date {
            self.document_date = dd;
        }
        if let Some(r) = reference {
            self.reference = Some(r);
        }
        if let Some(l) = lines {
            if l.is_empty() {
                return Err(JournalEntryError::EmptyLines);
            }
            self.lines = l;
        }

        Ok(())
    }

    // ============================================================================
    // 特殊总账业务方法 (UMSKZ)
    // ============================================================================

    /// 判断凭证是否包含特殊总账行项目
    pub fn has_special_gl_items(&self) -> bool {
        self.lines.iter().any(|line| line.is_special_gl())
    }

    /// 获取所有特殊总账行项目
    pub fn get_special_gl_items(&self) -> Vec<&LineItem> {
        self.lines.iter().filter(|line| line.is_special_gl()).collect()
    }

    /// 获取所有预付款行项目
    pub fn get_down_payment_items(&self) -> Vec<&LineItem> {
        self.lines.iter().filter(|line| line.is_down_payment()).collect()
    }

    /// 获取所有预收款行项目
    pub fn get_advance_payment_items(&self) -> Vec<&LineItem> {
        self.lines.iter().filter(|line| line.is_advance_payment()).collect()
    }

    /// 获取所有票据相关行项目
    pub fn get_bill_related_items(&self) -> Vec<&LineItem> {
        self.lines.iter().filter(|line| line.is_bill_related()).collect()
    }

    /// 计算特殊总账行项目的总金额
    pub fn calculate_special_gl_amount(&self, special_gl_type: SpecialGlType) -> Decimal {
        self.lines
            .iter()
            .filter(|line| line.special_gl_indicator == special_gl_type)
            .map(|line| line.local_amount)
            .sum()
    }

    /// 计算预付款总额
    pub fn calculate_down_payment_amount(&self) -> Decimal {
        self.calculate_special_gl_amount(SpecialGlType::DownPayment)
    }

    /// 计算预收款总额
    pub fn calculate_advance_payment_amount(&self) -> Decimal {
        self.calculate_special_gl_amount(SpecialGlType::AdvancePayment)
    }

    /// 计算票据总额
    pub fn calculate_bill_amount(&self) -> Decimal {
        self.calculate_special_gl_amount(SpecialGlType::BillOfExchange)
    }

    /// 按特殊总账类型分组统计
    pub fn group_by_special_gl_type(&self) -> std::collections::HashMap<SpecialGlType, Vec<&LineItem>> {
        let mut map = std::collections::HashMap::new();
        for line in &self.lines {
            map.entry(line.special_gl_indicator)
                .or_insert_with(Vec::new)
                .push(line);
        }
        map
    }

    /// 获取特殊总账类型摘要
    pub fn get_special_gl_summary(&self) -> Vec<(SpecialGlType, usize, Decimal)> {
        let grouped = self.group_by_special_gl_type();
        let mut summary: Vec<(SpecialGlType, usize, Decimal)> = grouped
            .into_iter()
            .map(|(gl_type, items)| {
                let count = items.len();
                let total: Decimal = items.iter().map(|item| item.local_amount).sum();
                (gl_type, count, total)
            })
            .collect();

        // 按类型排序
        summary.sort_by_key(|(gl_type, _, _)| format!("{:?}", gl_type));
        summary
    }

    /// 验证特殊总账业务规则
    pub fn validate_special_gl_rules(&self) -> Result<(), String> {
        for line in &self.lines {
            // 规则 1: 特殊总账行项目应该有业务伙伴信息（在实际应用中）
            // 这里只是示例，实际验证需要根据业务需求调整
            if line.is_special_gl() {
                // 可以添加更多验证规则
                // 例如：票据必须有到期日、预付款必须有供应商等
            }
        }
        Ok(())
    }

    /// 判断是否为纯特殊总账凭证（所有行项目都是特殊总账）
    pub fn is_pure_special_gl_entry(&self) -> bool {
        !self.lines.is_empty() && self.lines.iter().all(|line| line.is_special_gl())
    }

    /// 判断是否为混合凭证（既有特殊总账又有普通业务）
    pub fn is_mixed_entry(&self) -> bool {
        let has_special = self.lines.iter().any(|line| line.is_special_gl());
        let has_normal = self.lines.iter().any(|line| !line.is_special_gl());
        has_special && has_normal
    }

    /// 获取凭证的特殊总账类型列表（去重）
    pub fn get_special_gl_types(&self) -> Vec<SpecialGlType> {
        let mut types: Vec<SpecialGlType> = self.lines
            .iter()
            .filter(|line| line.is_special_gl())
            .map(|line| line.special_gl_indicator)
            .collect();
        types.sort_by_key(|t| format!("{:?}", t));
        types.dedup();
        types
    }
}


// Implement Entity trait
impl cuba_core::domain::Entity for JournalEntry {
    type Id = Uuid;
    fn id(&self) -> &Self::Id {
        &self.id
    }
}

// Implement AggregateRoot marker trait
impl AggregateRoot for JournalEntry {}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parallel_accounting_basic() {
        // 测试创建多分类账凭证
        let line_0l_debit = LineItem::new(
            1,
            "1000".to_string(),
            DebitCredit::Debit,
            dec!(1000.00),
            dec!(1000.00),
        );

        let line_0l_credit = LineItem::new(
            2,
            "2000".to_string(),
            DebitCredit::Credit,
            dec!(1000.00),
            dec!(1000.00),
        );

        // IFRS 分类账 (1L)
        let line_1l_debit = LineItem::with_ledger(
            3,
            "1000".to_string(),
            DebitCredit::Debit,
            dec!(1000.00),
            dec!(1000.00),
            "1L".to_string(),
            LedgerType::NonLeading,
        );

        let line_1l_credit = LineItem::with_ledger(
            4,
            "2000".to_string(),
            DebitCredit::Credit,
            dec!(1000.00),
            dec!(1000.00),
            "1L".to_string(),
            LedgerType::NonLeading,
        );

        // 验证分类账字段
        assert_eq!(line_0l_debit.ledger, "0L");
        assert_eq!(line_0l_debit.ledger_type, LedgerType::Leading);
        assert_eq!(line_0l_debit.ledger_amount, None);

        assert_eq!(line_1l_debit.ledger, "1L");
        assert_eq!(line_1l_debit.ledger_type, LedgerType::NonLeading);
        assert_eq!(line_1l_debit.ledger_amount, Some(dec!(1000.00)));

        // 创建包含多分类账的凭证
        let entry = JournalEntry::new(
            "1000".to_string(),
            2026,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            "USD".to_string(),
            Some("Parallel accounting test".to_string()),
            vec![line_0l_debit, line_0l_credit, line_1l_debit, line_1l_credit],
            Some("tenant1".to_string()),
        );

        assert!(entry.is_ok());
        let entry = entry.unwrap();
        assert_eq!(entry.lines.len(), 4);
    }

    #[test]
    fn test_parallel_accounting_balance_per_ledger() {
        // 测试每个分类账的借贷平衡
        let lines = vec![
            // 主分类账 0L
            LineItem::new(1, "1000".to_string(), DebitCredit::Debit, dec!(1000.00), dec!(1000.00)),
            LineItem::new(2, "2000".to_string(), DebitCredit::Credit, dec!(1000.00), dec!(1000.00)),
            // IFRS 分类账 1L
            LineItem::with_ledger(
                3,
                "1000".to_string(),
                DebitCredit::Debit,
                dec!(1000.00),
                dec!(1000.00),
                "1L".to_string(),
                LedgerType::NonLeading,
            ),
            LineItem::with_ledger(
                4,
                "2000".to_string(),
                DebitCredit::Credit,
                dec!(1000.00),
                dec!(1000.00),
                "1L".to_string(),
                LedgerType::NonLeading,
            ),
        ];

        let entry = JournalEntry::new(
            "1000".to_string(),
            2026,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            "USD".to_string(),
            None,
            lines,
            None,
        ).unwrap();

        // 验证整体平衡
        assert!(entry.validate_balance().is_ok());

        // 验证每个分类账的平衡
        let ledger_0l_lines: Vec<&LineItem> = entry.lines.iter().filter(|l| l.ledger == "0L").collect();
        let ledger_1l_lines: Vec<&LineItem> = entry.lines.iter().filter(|l| l.ledger == "1L").collect();

        assert_eq!(ledger_0l_lines.len(), 2);
        assert_eq!(ledger_1l_lines.len(), 2);

        // 验证 0L 平衡
        let debit_0l: Decimal = ledger_0l_lines.iter()
            .filter(|l| l.debit_credit == DebitCredit::Debit)
            .map(|l| l.amount)
            .sum();
        let credit_0l: Decimal = ledger_0l_lines.iter()
            .filter(|l| l.debit_credit == DebitCredit::Credit)
            .map(|l| l.amount)
            .sum();
        assert_eq!(debit_0l, credit_0l);

        // 验证 1L 平衡
        let debit_1l: Decimal = ledger_1l_lines.iter()
            .filter(|l| l.debit_credit == DebitCredit::Debit)
            .map(|l| l.amount)
            .sum();
        let credit_1l: Decimal = ledger_1l_lines.iter()
            .filter(|l| l.debit_credit == DebitCredit::Credit)
            .map(|l| l.amount)
            .sum();
        assert_eq!(debit_1l, credit_1l);
    }

    #[test]
    fn test_parallel_accounting_different_amounts() {
        // 测试不同分类账使用不同金额（例如不同会计准则下的估值差异）
        let lines = vec![
            // 主分类账 0L - 本地 GAAP
            LineItem::new(1, "1000".to_string(), DebitCredit::Debit, dec!(1000.00), dec!(1000.00)),
            LineItem::new(2, "2000".to_string(), DebitCredit::Credit, dec!(1000.00), dec!(1000.00)),
            // IFRS 分类账 1L - 使用不同的估值
            LineItem::with_ledger(
                3,
                "1000".to_string(),
                DebitCredit::Debit,
                dec!(1100.00),  // IFRS 估值更高
                dec!(1100.00),
                "1L".to_string(),
                LedgerType::NonLeading,
            ),
            LineItem::with_ledger(
                4,
                "2000".to_string(),
                DebitCredit::Credit,
                dec!(1100.00),
                dec!(1100.00),
                "1L".to_string(),
                LedgerType::NonLeading,
            ),
        ];

        let entry = JournalEntry::new(
            "1000".to_string(),
            2026,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            "USD".to_string(),
            Some("Different valuation per ledger".to_string()),
            lines,
            None,
        ).unwrap();

        // 验证整体平衡（所有行项目）
        assert!(entry.validate_balance().is_ok());

        // 验证不同分类账有不同的金额
        let ledger_0l_total: Decimal = entry.lines.iter()
            .filter(|l| l.ledger == "0L" && l.debit_credit == DebitCredit::Debit)
            .map(|l| l.amount)
            .sum();
        let ledger_1l_total: Decimal = entry.lines.iter()
            .filter(|l| l.ledger == "1L" && l.debit_credit == DebitCredit::Debit)
            .map(|l| l.amount)
            .sum();

        assert_eq!(ledger_0l_total, dec!(1000.00));
        assert_eq!(ledger_1l_total, dec!(1100.00));
    }

    #[test]
    fn test_parallel_accounting_multiple_ledgers() {
        // 测试多个分类账 (0L, 1L, 2L)
        let lines = vec![
            // 主分类账 0L
            LineItem::new(1, "1000".to_string(), DebitCredit::Debit, dec!(1000.00), dec!(1000.00)),
            LineItem::new(2, "2000".to_string(), DebitCredit::Credit, dec!(1000.00), dec!(1000.00)),
            // IFRS 分类账 1L
            LineItem::with_ledger(
                3,
                "1000".to_string(),
                DebitCredit::Debit,
                dec!(1000.00),
                dec!(1000.00),
                "1L".to_string(),
                LedgerType::NonLeading,
            ),
            LineItem::with_ledger(
                4,
                "2000".to_string(),
                DebitCredit::Credit,
                dec!(1000.00),
                dec!(1000.00),
                "1L".to_string(),
                LedgerType::NonLeading,
            ),
            // US GAAP 分类账 2L
            LineItem::with_ledger(
                5,
                "1000".to_string(),
                DebitCredit::Debit,
                dec!(950.00),
                dec!(950.00),
                "2L".to_string(),
                LedgerType::NonLeading,
            ),
            LineItem::with_ledger(
                6,
                "2000".to_string(),
                DebitCredit::Credit,
                dec!(950.00),
                dec!(950.00),
                "2L".to_string(),
                LedgerType::NonLeading,
            ),
        ];

        let entry = JournalEntry::new(
            "1000".to_string(),
            2026,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            "USD".to_string(),
            Some("Multiple ledgers test".to_string()),
            lines,
            None,
        ).unwrap();

        assert_eq!(entry.lines.len(), 6);

        // 验证每个分类账都有正确的行数
        let ledger_0l_count = entry.lines.iter().filter(|l| l.ledger == "0L").count();
        let ledger_1l_count = entry.lines.iter().filter(|l| l.ledger == "1L").count();
        let ledger_2l_count = entry.lines.iter().filter(|l| l.ledger == "2L").count();

        assert_eq!(ledger_0l_count, 2);
        assert_eq!(ledger_1l_count, 2);
        assert_eq!(ledger_2l_count, 2);
    }

    #[test]
    fn test_ledger_type_conversion() {
        // 测试 LedgerType 的转换
        assert_eq!(LedgerType::from(1), LedgerType::Leading);
        assert_eq!(LedgerType::from(2), LedgerType::NonLeading);
        assert_eq!(LedgerType::from(3), LedgerType::Extension);
        assert_eq!(LedgerType::from(999), LedgerType::Leading); // 默认值

        assert_eq!(i32::from(LedgerType::Leading), 1);
        assert_eq!(i32::from(LedgerType::NonLeading), 2);
        assert_eq!(i32::from(LedgerType::Extension), 3);
    }

    #[test]
    fn test_parallel_accounting_with_reversal() {
        // 测试并行会计的冲销功能
        let lines = vec![
            LineItem::new(1, "1000".to_string(), DebitCredit::Debit, dec!(1000.00), dec!(1000.00)),
            LineItem::new(2, "2000".to_string(), DebitCredit::Credit, dec!(1000.00), dec!(1000.00)),
            LineItem::with_ledger(
                3,
                "1000".to_string(),
                DebitCredit::Debit,
                dec!(1000.00),
                dec!(1000.00),
                "1L".to_string(),
                LedgerType::NonLeading,
            ),
            LineItem::with_ledger(
                4,
                "2000".to_string(),
                DebitCredit::Credit,
                dec!(1000.00),
                dec!(1000.00),
                "1L".to_string(),
                LedgerType::NonLeading,
            ),
        ];

        let mut entry = JournalEntry::new(
            "1000".to_string(),
            2026,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            "USD".to_string(),
            None,
            lines,
            None,
        ).unwrap();

        // 过账原凭证
        entry.post("DOC-001".to_string()).unwrap();

        // 创建冲销凭证
        let reversal = entry.create_reversal_entry(
            NaiveDate::from_ymd_opt(2026, 1, 19).unwrap()
        ).unwrap();

        // 验证冲销凭证的行数与原凭证相同
        assert_eq!(reversal.lines.len(), entry.lines.len());

        // 验证冲销凭证已过账
        assert_eq!(reversal.status, PostingStatus::Posted);
    }

    #[test]
    fn test_default_ledger_values() {
        // 测试默认分类账值
        let line = LineItem::new(
            1,
            "1000".to_string(),
            DebitCredit::Debit,
            dec!(100.00),
            dec!(100.00),
        );

        assert_eq!(line.ledger, "0L");
        assert_eq!(line.ledger_type, LedgerType::Leading);
        assert_eq!(line.ledger_amount, None);
    }

    // ============================================================================
    // 特殊总账标识 (UMSKZ) 测试
    // ============================================================================

    #[test]
    fn test_special_gl_type_conversion() {
        // 测试 SAP 代码转换
        assert_eq!(SpecialGlType::Normal.to_sap_code(), "");
        assert_eq!(SpecialGlType::BillOfExchange.to_sap_code(), "A");
        assert_eq!(SpecialGlType::DownPayment.to_sap_code(), "F");
        assert_eq!(SpecialGlType::AdvancePayment.to_sap_code(), "V");
        assert_eq!(SpecialGlType::BillDiscount.to_sap_code(), "W");

        // 测试从 SAP 代码转换
        assert_eq!(SpecialGlType::from_sap_code(""), SpecialGlType::Normal);
        assert_eq!(SpecialGlType::from_sap_code("A"), SpecialGlType::BillOfExchange);
        assert_eq!(SpecialGlType::from_sap_code("F"), SpecialGlType::DownPayment);
        assert_eq!(SpecialGlType::from_sap_code("V"), SpecialGlType::AdvancePayment);
        assert_eq!(SpecialGlType::from_sap_code("W"), SpecialGlType::BillDiscount);
        assert_eq!(SpecialGlType::from_sap_code("X"), SpecialGlType::Normal); // 无效值默认为 Normal
    }

    #[test]
    fn test_special_gl_type_description() {
        // 测试描述
        assert_eq!(SpecialGlType::Normal.description(), "普通业务");
        assert_eq!(SpecialGlType::BillOfExchange.description(), "票据 (Bills of Exchange)");
        assert_eq!(SpecialGlType::DownPayment.description(), "预付款 (Down Payment)");
        assert_eq!(SpecialGlType::AdvancePayment.description(), "预收款 (Advance Payment)");
        assert_eq!(SpecialGlType::BillDiscount.description(), "票据贴现 (Bill of Exchange Discount)");
    }

    #[test]
    fn test_special_gl_type_default() {
        // 测试默认值
        let default_type = SpecialGlType::default();
        assert_eq!(default_type, SpecialGlType::Normal);
    }

    #[test]
    fn test_line_item_with_special_gl() {
        // 测试创建特殊总账行项目
        let line = LineItem::with_special_gl(
            1,
            "1100".to_string(),
            DebitCredit::Debit,
            dec!(10000.00),
            dec!(10000.00),
            SpecialGlType::DownPayment,
        );

        assert_eq!(line.line_number, 1);
        assert_eq!(line.account_id, "1100");
        assert_eq!(line.debit_credit, DebitCredit::Debit);
        assert_eq!(line.amount, dec!(10000.00));
        assert_eq!(line.special_gl_indicator, SpecialGlType::DownPayment);
        assert_eq!(line.ledger, "0L");
        assert_eq!(line.ledger_type, LedgerType::Leading);
    }

    #[test]
    fn test_down_payment_journal_entry() {
        // 测试创建预付款凭证
        let lines = vec![
            LineItem::with_special_gl(
                1,
                "1100".to_string(),
                DebitCredit::Debit,
                dec!(10000.00),
                dec!(10000.00),
                SpecialGlType::DownPayment,
            ),
            LineItem::new(
                2,
                "2100".to_string(),
                DebitCredit::Credit,
                dec!(10000.00),
                dec!(10000.00),
            ),
        ];

        let entry = JournalEntry::new(
            "1000".to_string(),
            2026,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            "CNY".to_string(),
            Some("预付款给供应商".to_string()),
            lines,
            Some("tenant1".to_string()),
        ).unwrap();

        assert_eq!(entry.lines.len(), 2);
        assert_eq!(entry.lines[0].special_gl_indicator, SpecialGlType::DownPayment);
        assert_eq!(entry.lines[1].special_gl_indicator, SpecialGlType::Normal);

        // 验证借贷平衡
        assert!(entry.validate_balance().is_ok());
    }

    #[test]
    fn test_bill_of_exchange_journal_entry() {
        // 测试创建票据凭证
        let lines = vec![
            LineItem::with_special_gl(
                1,
                "1120".to_string(),
                DebitCredit::Debit,
                dec!(50000.00),
                dec!(50000.00),
                SpecialGlType::BillOfExchange,
            ),
            LineItem::new(
                2,
                "4000".to_string(),
                DebitCredit::Credit,
                dec!(50000.00),
                dec!(50000.00),
            ),
        ];

        let entry = JournalEntry::new(
            "1000".to_string(),
            2026,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            "CNY".to_string(),
            Some("应收票据".to_string()),
            lines,
            None,
        ).unwrap();

        assert_eq!(entry.lines[0].special_gl_indicator, SpecialGlType::BillOfExchange);
        assert!(entry.validate_balance().is_ok());
    }

    #[test]
    fn test_advance_payment_journal_entry() {
        // 测试创建预收款凭证
        let lines = vec![
            LineItem::new(
                1,
                "2100".to_string(),
                DebitCredit::Debit,
                dec!(20000.00),
                dec!(20000.00),
            ),
            LineItem::with_special_gl(
                2,
                "2200".to_string(),
                DebitCredit::Credit,
                dec!(20000.00),
                dec!(20000.00),
                SpecialGlType::AdvancePayment,
            ),
        ];

        let entry = JournalEntry::new(
            "1000".to_string(),
            2026,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            "CNY".to_string(),
            Some("预收客户款项".to_string()),
            lines,
            None,
        ).unwrap();

        assert_eq!(entry.lines[1].special_gl_indicator, SpecialGlType::AdvancePayment);
        assert!(entry.validate_balance().is_ok());
    }

    #[test]
    fn test_special_gl_with_reversal() {
        // 测试特殊总账凭证的冲销
        let lines = vec![
            LineItem::with_special_gl(
                1,
                "1100".to_string(),
                DebitCredit::Debit,
                dec!(10000.00),
                dec!(10000.00),
                SpecialGlType::DownPayment,
            ),
            LineItem::new(
                2,
                "2100".to_string(),
                DebitCredit::Credit,
                dec!(10000.00),
                dec!(10000.00),
            ),
        ];

        let mut entry = JournalEntry::new(
            "1000".to_string(),
            2026,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            "CNY".to_string(),
            None,
            lines,
            None,
        ).unwrap();

        // 过账原凭证
        entry.post("DOC-001".to_string()).unwrap();

        // 创建冲销凭证
        let reversal = entry.create_reversal_entry(
            NaiveDate::from_ymd_opt(2026, 1, 19).unwrap()
        ).unwrap();

        // 验证冲销凭证保留了特殊总账标识
        assert_eq!(reversal.lines.len(), 2);
        assert_eq!(reversal.lines[0].special_gl_indicator, SpecialGlType::DownPayment);
        assert_eq!(reversal.lines[1].special_gl_indicator, SpecialGlType::Normal);

        // 验证借贷方向已反转
        assert_eq!(reversal.lines[0].debit_credit, DebitCredit::Credit);
        assert_eq!(reversal.lines[1].debit_credit, DebitCredit::Debit);

        // 验证冲销凭证已过账
        assert_eq!(reversal.status, PostingStatus::Posted);
    }

    #[test]
    fn test_mixed_special_gl_types() {
        // 测试混合特殊总账类型的凭证
        let lines = vec![
            LineItem::with_special_gl(
                1,
                "1100".to_string(),
                DebitCredit::Debit,
                dec!(5000.00),
                dec!(5000.00),
                SpecialGlType::DownPayment,
            ),
            LineItem::with_special_gl(
                2,
                "1120".to_string(),
                DebitCredit::Debit,
                dec!(3000.00),
                dec!(3000.00),
                SpecialGlType::BillOfExchange,
            ),
            LineItem::new(
                3,
                "2100".to_string(),
                DebitCredit::Credit,
                dec!(8000.00),
                dec!(8000.00),
            ),
        ];

        let entry = JournalEntry::new(
            "1000".to_string(),
            2026,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            "CNY".to_string(),
            Some("混合特殊总账业务".to_string()),
            lines,
            None,
        ).unwrap();

        assert_eq!(entry.lines.len(), 3);
        assert_eq!(entry.lines[0].special_gl_indicator, SpecialGlType::DownPayment);
        assert_eq!(entry.lines[1].special_gl_indicator, SpecialGlType::BillOfExchange);
        assert_eq!(entry.lines[2].special_gl_indicator, SpecialGlType::Normal);
        assert!(entry.validate_balance().is_ok());
    }

    #[test]
    fn test_special_gl_with_parallel_accounting() {
        // 测试特殊总账 + 并行会计
        let lines = vec![
            // 主分类账 0L - 预付款
            LineItem::with_special_gl(
                1,
                "1100".to_string(),
                DebitCredit::Debit,
                dec!(10000.00),
                dec!(10000.00),
                SpecialGlType::DownPayment,
            ),
            LineItem::new(
                2,
                "2100".to_string(),
                DebitCredit::Credit,
                dec!(10000.00),
                dec!(10000.00),
            ),
            // IFRS 分类账 1L - 预付款
            {
                let mut line = LineItem::with_ledger(
                    3,
                    "1100".to_string(),
                    DebitCredit::Debit,
                    dec!(10000.00),
                    dec!(10000.00),
                    "1L".to_string(),
                    LedgerType::NonLeading,
                );
                line.special_gl_indicator = SpecialGlType::DownPayment;
                line
            },
            LineItem::with_ledger(
                4,
                "2100".to_string(),
                DebitCredit::Credit,
                dec!(10000.00),
                dec!(10000.00),
                "1L".to_string(),
                LedgerType::NonLeading,
            ),
        ];

        let entry = JournalEntry::new(
            "1000".to_string(),
            2026,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            "CNY".to_string(),
            Some("预付款 - 并行会计".to_string()),
            lines,
            None,
        ).unwrap();

        assert_eq!(entry.lines.len(), 4);

        // 验证主分类账的特殊总账标识
        let ledger_0l_lines: Vec<&LineItem> = entry.lines.iter()
            .filter(|l| l.ledger == "0L")
            .collect();
        assert_eq!(ledger_0l_lines.len(), 2);
        assert_eq!(ledger_0l_lines[0].special_gl_indicator, SpecialGlType::DownPayment);

        // 验证 IFRS 分类账的特殊总账标识
        let ledger_1l_lines: Vec<&LineItem> = entry.lines.iter()
            .filter(|l| l.ledger == "1L")
            .collect();
        assert_eq!(ledger_1l_lines.len(), 2);
        assert_eq!(ledger_1l_lines[0].special_gl_indicator, SpecialGlType::DownPayment);

        // 验证借贷平衡
        assert!(entry.validate_balance().is_ok());
    }

    #[test]
    fn test_special_gl_type_serialization() {
        // 测试序列化和反序列化
        let line = LineItem::with_special_gl(
            1,
            "1100".to_string(),
            DebitCredit::Debit,
            dec!(10000.00),
            dec!(10000.00),
            SpecialGlType::DownPayment,
        );

        // 序列化
        let json = serde_json::to_string(&line).unwrap();
        assert!(json.contains("DownPayment"));

        // 反序列化
        let deserialized: LineItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.special_gl_indicator, SpecialGlType::DownPayment);
        assert_eq!(deserialized.amount, dec!(10000.00));
    }

    // ============================================================================
    // 付款执行字段 (Payment Execution) 测试
    // ============================================================================

    #[test]
    fn test_payment_execution_detail_creation() {
        // 测试创建付款执行详细信息
        let payment_exec = PaymentExecutionDetail::new("T".to_string());

        assert_eq!(payment_exec.payment_method, "T");
        assert_eq!(payment_exec.house_bank, None);
        assert_eq!(payment_exec.partner_bank_type, None);
        assert_eq!(payment_exec.payment_block, None);
        assert!(!payment_exec.is_blocked());
    }

    #[test]
    fn test_payment_execution_with_details() {
        // 测试创建完整的付款执行信息
        let payment_exec = PaymentExecutionDetail::with_details(
            "W".to_string(),
            Some("BANK001".to_string()),
            Some("SWIFT".to_string()),
        );

        assert_eq!(payment_exec.payment_method, "W");
        assert_eq!(payment_exec.house_bank, Some("BANK001".to_string()));
        assert_eq!(payment_exec.partner_bank_type, Some("SWIFT".to_string()));
    }

    #[test]
    fn test_payment_execution_with_block() {
        // 测试付款冻结
        let payment_exec = PaymentExecutionDetail::new("T".to_string())
            .with_payment_block("A".to_string());

        assert!(payment_exec.is_blocked());
        assert_eq!(payment_exec.payment_block, Some("A".to_string()));
    }

    #[test]
    fn test_payment_execution_with_priority() {
        // 测试付款优先级
        let payment_exec = PaymentExecutionDetail::new("T".to_string())
            .with_priority(1);

        assert_eq!(payment_exec.payment_priority, Some(1));
    }

    #[test]
    fn test_payment_execution_method_description() {
        // 测试付款方式描述
        let payment_exec_t = PaymentExecutionDetail::new("T".to_string());
        assert_eq!(payment_exec_t.payment_method_description(), "银行转账 (Bank Transfer)");

        let payment_exec_c = PaymentExecutionDetail::new("C".to_string());
        assert_eq!(payment_exec_c.payment_method_description(), "支票 (Check)");

        let payment_exec_w = PaymentExecutionDetail::new("W".to_string());
        assert_eq!(payment_exec_w.payment_method_description(), "电汇 (Wire Transfer)");
    }

    #[test]
    fn test_payment_execution_validation() {
        // 测试验证付款执行信息
        let valid_payment = PaymentExecutionDetail::new("T".to_string())
            .with_priority(5);
        assert!(valid_payment.validate().is_ok());

        // 测试无效优先级
        let invalid_payment = PaymentExecutionDetail::new("T".to_string())
            .with_priority(10);
        assert!(invalid_payment.validate().is_err());
    }

    #[test]
    fn test_line_item_with_payment_execution() {
        // 测试创建带付款执行信息的行项目
        let payment_exec = PaymentExecutionDetail::with_details(
            "T".to_string(),
            Some("BANK001".to_string()),
            Some("SWIFT".to_string()),
        );

        let line = LineItem::new(
            1,
            "2100".to_string(),
            DebitCredit::Credit,
            dec!(50000.00),
            dec!(50000.00),
        ).with_payment_execution(payment_exec.clone());

        assert!(line.payment_execution.is_some());
        let exec = line.payment_execution.unwrap();
        assert_eq!(exec.payment_method, "T");
        assert_eq!(exec.house_bank, Some("BANK001".to_string()));
    }

    #[test]
    fn test_line_item_builder_with_payment_execution() {
        // 测试使用构建器创建带付款执行信息的行项目
        let payment_exec = PaymentExecutionDetail::new("W".to_string())
            .with_priority(1)
            .with_reference("PAY-2026-001".to_string());

        let line = LineItem::builder()
            .line_number(1)
            .account_id("2100".to_string())
            .debit_credit(DebitCredit::Credit)
            .amount(dec!(100000.00))
            .local_amount(dec!(100000.00))
            .payment_execution(payment_exec)
            .build()
            .unwrap();

        assert!(line.payment_execution.is_some());
        let exec = line.payment_execution.unwrap();
        assert_eq!(exec.payment_method, "W");
        assert_eq!(exec.payment_priority, Some(1));
        assert_eq!(exec.payment_reference, Some("PAY-2026-001".to_string()));
    }

    #[test]
    fn test_accounts_payable_with_payment_execution() {
        // 测试应付账款凭证带付款执行信息
        let payment_exec = PaymentExecutionDetail::with_details(
            "T".to_string(),
            Some("BANK001".to_string()),
            None,
        ).with_baseline_date(NaiveDate::from_ymd_opt(2026, 1, 18).unwrap())
          .with_priority(2);

        let lines = vec![
            LineItem::new(
                1,
                "5000".to_string(),
                DebitCredit::Debit,
                dec!(100000.00),
                dec!(100000.00),
            ),
            LineItem::new(
                2,
                "2100".to_string(),
                DebitCredit::Credit,
                dec!(100000.00),
                dec!(100000.00),
            ).with_payment_execution(payment_exec),
        ];

        let entry = JournalEntry::new(
            "1000".to_string(),
            2026,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
            "CNY".to_string(),
            Some("采购发票 - 带付款执行信息".to_string()),
            lines,
            None,
        ).unwrap();

        assert_eq!(entry.lines.len(), 2);
        assert!(entry.lines[1].payment_execution.is_some());

        let exec = entry.lines[1].payment_execution.as_ref().unwrap();
        assert_eq!(exec.payment_method, "T");
        assert_eq!(exec.house_bank, Some("BANK001".to_string()));
        assert_eq!(exec.payment_priority, Some(2));

        // 验证借贷平衡
        assert!(entry.validate_balance().is_ok());
    }

    #[test]
    fn test_payment_execution_with_block_scenario() {
        // 测试付款冻结场景
        let payment_exec = PaymentExecutionDetail::new("T".to_string())
            .with_payment_block("A".to_string()); // A = 争议

        let line = LineItem::new(
            1,
            "2100".to_string(),
            DebitCredit::Credit,
            dec!(50000.00),
            dec!(50000.00),
        ).with_payment_execution(payment_exec);

        assert!(line.payment_execution.is_some());
        let exec = line.payment_execution.unwrap();
        assert!(exec.is_blocked());
        assert_eq!(exec.payment_block, Some("A".to_string()));
    }

    #[test]
    fn test_payment_execution_serialization() {
        // 测试付款执行信息的序列化和反序列化
        let payment_exec = PaymentExecutionDetail::with_details(
            "W".to_string(),
            Some("BANK001".to_string()),
            Some("SWIFT".to_string()),
        ).with_priority(1);

        let line = LineItem::new(
            1,
            "2100".to_string(),
            DebitCredit::Credit,
            dec!(50000.00),
            dec!(50000.00),
        ).with_payment_execution(payment_exec);

        // 序列化
        let json = serde_json::to_string(&line).unwrap();
        assert!(json.contains("payment_execution"));
        assert!(json.contains("BANK001"));

        // 反序列化
        let deserialized: LineItem = serde_json::from_str(&json).unwrap();
        assert!(deserialized.payment_execution.is_some());
        let exec = deserialized.payment_execution.unwrap();
        assert_eq!(exec.payment_method, "W");
        assert_eq!(exec.house_bank, Some("BANK001".to_string()));
    }
}

