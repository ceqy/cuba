// ============================================================================
// Universal Journal Entry Domain Model
// 描述: Universal Journal (ACDOCA) 领域模型
// ============================================================================

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 枚举定义
// ============================================================================

/// 来源模块标识
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceModule {
    Unspecified,
    GL,  // 总账
    AP,  // 应付账款
    AR,  // 应收账款
    AA,  // 固定资产
    MM,  // 物料管理
    SD,  // 销售与分销
    CO,  // 成本控制
    TR,  // 资金管理
}

/// 账户类型 (KOART)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Unspecified,
    GL,       // D - 总账科目
    Customer, // D - 客户
    Vendor,   // K - 供应商
    Asset,    // A - 固定资产
    Material, // M - 物料
}

// ============================================================================
// Universal Journal Entry (ACDOCA 完整映射)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UniversalJournalEntry {
    // ============================================================================
    // 主键字段 (Primary Key Fields)
    // ============================================================================
    pub ledger: String,                    // RLDNR 分类账
    pub company_code: String,              // RBUKRS 公司代码
    pub fiscal_year: i32,                  // GJAHR 会计年度
    pub document_number: String,           // BELNR 凭证号
    pub document_line: i32,                // DOCLN 凭证行号

    // ============================================================================
    // 凭证抬头字段 (Document Header Fields)
    // ============================================================================
    pub document_type: String,             // BLART 凭证类型
    pub document_date: NaiveDate,          // BLDAT 凭证日期
    pub posting_date: NaiveDate,           // BUDAT 过账日期
    pub fiscal_period: i32,                // MONAT 会计期间
    pub reference_document: Option<String>, // XBLNR 参考凭证号
    pub header_text: Option<String>,       // BKTXT 凭证抬头文本
    pub document_currency: String,         // WAERS 凭证货币
    pub exchange_rate: Option<Decimal>,    // KURSF 汇率
    pub logical_system: Option<String>,    // AWSYS 逻辑系统
    pub transaction_code: Option<String>,  // TCODE 事务代码

    // ============================================================================
    // 行项目字段 (Line Item Fields)
    // ============================================================================
    pub posting_key: String,               // BSCHL 过账码
    pub debit_credit_indicator: String,    // SHKZG 借贷标识（S-借方，H-贷方）
    pub account_type: AccountType,         // KOART 账户类型
    pub gl_account: String,                // RACCT 总账科目
    pub business_partner: Option<String>,  // KUNNR/LIFNR 业务伙伴
    pub material: Option<String>,          // MATNR 物料号
    pub plant: Option<String>,             // WERKS 工厂
    pub item_text: Option<String>,         // SGTXT 行项目文本
    pub assignment_number: Option<String>, // ZUONR 指派编号

    // ============================================================================
    // 金额字段 (Amount Fields)
    // ============================================================================
    pub amount_in_document_currency: Decimal, // WRBTR 凭证货币金额
    pub amount_in_local_currency: Decimal,    // DMBTR 本位币金额
    pub amount_in_group_currency: Option<Decimal>, // DMBE2 集团货币金额
    pub amount_in_global_currency: Option<Decimal>, // DMBE3 全球货币金额
    pub amount_in_ledger_currency: Option<Decimal>, // HSL 分类账货币金额

    // ============================================================================
    // 数量字段 (Quantity Fields)
    // ============================================================================
    pub quantity: Option<Decimal>,         // MENGE 数量
    pub quantity_unit: Option<String>,     // MEINS 单位

    // ============================================================================
    // 成本对象字段 (Cost Object Fields)
    // ============================================================================
    pub cost_center: Option<String>,       // KOSTL 成本中心
    pub profit_center: Option<String>,     // PRCTR 利润中心
    pub segment: Option<String>,           // SEGMENT 段
    pub functional_area: Option<String>,   // FKBER 功能范围
    pub business_area: Option<String>,     // GSBER 业务范围
    pub controlling_area: Option<String>,  // KOKRS 控制范围
    pub internal_order: Option<String>,    // AUFNR 内部订单
    pub wbs_element: Option<String>,       // PS_PSP_PNR WBS 元素
    pub sales_order: Option<String>,       // VBELN 销售订单
    pub sales_order_item: Option<i32>,     // POSNR 销售订单行项目

    // ============================================================================
    // 税务字段 (Tax Fields)
    // ============================================================================
    pub tax_code: Option<String>,          // MWSKZ 税码
    pub tax_jurisdiction: Option<String>,  // TXJCD 税收辖区
    pub tax_amount: Option<Decimal>,       // MWSTS 税额

    // ============================================================================
    // 清账字段 (Clearing Fields)
    // ============================================================================
    pub clearing_document: Option<String>, // AUGBL 清账凭证号
    pub clearing_date: Option<NaiveDate>,  // AUGDT 清账日期

    // ============================================================================
    // 付款字段 (Payment Fields)
    // ============================================================================
    pub baseline_date: Option<NaiveDate>,  // ZFBDT 基准日期
    pub due_date: Option<NaiveDate>,       // NETDT 到期日
    pub payment_terms: Option<String>,     // ZTERM 付款条件
    pub payment_method: Option<String>,    // ZLSCH 付款方式
    pub payment_block: Option<String>,     // ZLSPR 付款冻结
    pub house_bank: Option<String>,        // HBKID 内部银行账户

    // ============================================================================
    // 特殊总账字段 (Special G/L Fields)
    // ============================================================================
    pub special_gl_indicator: Option<String>, // UMSKZ 特殊总账标识

    // ============================================================================
    // 发票参考字段 (Invoice Reference Fields)
    // ============================================================================
    pub reference_document_number: Option<String>, // REBZG 参考凭证号
    pub reference_fiscal_year: Option<i32>,        // REBZJ 参考会计年度
    pub reference_line_item: Option<i32>,          // REBZZ 参考行项目号
    pub reference_document_type: Option<String>,   // REBZT 参考凭证类型

    // ============================================================================
    // 业务交易类型字段 (Transaction Type Fields)
    // ============================================================================
    pub transaction_type: Option<String>,          // VRGNG 业务交易类型
    pub reference_transaction_type: Option<String>, // AWTYP 参考交易类型
    pub reference_key_1: Option<String>,           // AWREF 参考键 1
    pub reference_key_2: Option<String>,           // AWORG 参考键 2
    pub reference_key_3: Option<String>,           // AWSYS 参考键 3

    // ============================================================================
    // 组织维度字段 (Organizational Dimensions)
    // ============================================================================
    pub financial_area: Option<String>,    // RFAREA 财务范围
    pub consolidation_unit: Option<String>, // RUNIT 合并单位
    pub partner_company: Option<String>,   // VBUND 伙伴公司代码
    pub trading_partner: Option<String>,   // VKORG 交易伙伴

    // ============================================================================
    // 多币种字段 (Multi-Currency Fields)
    // ============================================================================
    pub local_currency: String,            // RHCUR 本位币
    pub group_currency: Option<String>,    // RKCUR 集团货币
    pub global_currency: Option<String>,   // RTCUR 全球货币
    pub amount_in_object_currency: Option<Decimal>, // OSL 对象货币金额
    pub amount_in_profit_center_currency: Option<Decimal>, // VSL 利润中心货币金额

    // ============================================================================
    // 催款字段 (Dunning Fields)
    // ============================================================================
    pub dunning_key: Option<String>,       // MSCHL 催款码
    pub dunning_block: Option<String>,     // MANST 催款冻结
    pub last_dunning_date: Option<NaiveDate>, // MADAT 上次催款日期
    pub dunning_level: Option<i32>,        // 催款级别

    // ============================================================================
    // 付款条件详细字段 (Payment Terms Detail)
    // ============================================================================
    pub discount_days_1: Option<i32>,      // ZBD1T 第一个折扣天数
    pub discount_days_2: Option<i32>,      // ZBD2T 第二个折扣天数
    pub net_payment_days: Option<i32>,     // ZBD3T 净付款天数
    pub discount_percent_1: Option<Decimal>, // ZBD1P 第一个折扣百分比
    pub discount_percent_2: Option<Decimal>, // ZBD2P 第二个折扣百分比
    pub discount_amount: Option<Decimal>,  // SKFBT 现金折扣金额

    // ============================================================================
    // 内部交易字段 (Internal Trading Fields)
    // ============================================================================
    pub sending_cost_center: Option<String>, // SCNTR 发送成本中心
    pub partner_profit_center: Option<String>, // PPRCTR 伙伴利润中心
    pub sending_financial_area: Option<String>, // SFAREA 发送财务范围

    // ============================================================================
    // 科目分配字段 (Account Assignment Fields)
    // ============================================================================
    pub account_assignment: Option<String>, // KTOSL 科目分配

    // ============================================================================
    // 本地 GAAP 字段 (Local GAAP Fields)
    // ============================================================================
    pub local_account: Option<String>,     // LOKKT 本地科目
    pub data_source: Option<String>,       // HRKFT 数据来源

    // ============================================================================
    // 字段拆分字段 (Field Split Fields)
    // ============================================================================
    pub split_method: Option<String>,      // XSPLITMOD 拆分方法
    pub manual_split: bool,                // MANSP 手工拆分标识

    // ============================================================================
    // 审计字段 (Audit Fields)
    // ============================================================================
    pub created_by: String,                // USNAM 创建人
    pub created_at: NaiveDateTime,         // CPUDT 创建日期时间
    pub changed_by: Option<String>,        // AENAM 修改人
    pub changed_at: Option<NaiveDateTime>, // AEDAT 修改日期时间

    // ============================================================================
    // 来源模块标识 (Source Module Identifier)
    // ============================================================================
    pub source_module: SourceModule,       // 来源模块

    // ============================================================================
    // 扩展字段 (Extension Fields)
    // ============================================================================
    pub extension_fields: HashMap<String, String>, // 扩展字段
}

impl UniversalJournalEntry {
    /// 创建新的 Universal Journal Entry
    pub fn new(
        ledger: String,
        company_code: String,
        fiscal_year: i32,
        document_number: String,
        document_line: i32,
        document_type: String,
        document_date: NaiveDate,
        posting_date: NaiveDate,
        fiscal_period: i32,
        document_currency: String,
        local_currency: String,
        posting_key: String,
        debit_credit_indicator: String,
        account_type: AccountType,
        gl_account: String,
        amount_in_document_currency: Decimal,
        amount_in_local_currency: Decimal,
        source_module: SourceModule,
        created_by: String,
    ) -> Self {
        Self {
            ledger,
            company_code,
            fiscal_year,
            document_number,
            document_line,
            document_type,
            document_date,
            posting_date,
            fiscal_period,
            reference_document: None,
            header_text: None,
            document_currency,
            exchange_rate: None,
            logical_system: None,
            transaction_code: None,
            posting_key,
            debit_credit_indicator,
            account_type,
            gl_account,
            business_partner: None,
            material: None,
            plant: None,
            item_text: None,
            assignment_number: None,
            amount_in_document_currency,
            amount_in_local_currency,
            amount_in_group_currency: None,
            amount_in_global_currency: None,
            amount_in_ledger_currency: None,
            quantity: None,
            quantity_unit: None,
            cost_center: None,
            profit_center: None,
            segment: None,
            functional_area: None,
            business_area: None,
            controlling_area: None,
            internal_order: None,
            wbs_element: None,
            sales_order: None,
            sales_order_item: None,
            tax_code: None,
            tax_jurisdiction: None,
            tax_amount: None,
            clearing_document: None,
            clearing_date: None,
            baseline_date: None,
            due_date: None,
            payment_terms: None,
            payment_method: None,
            payment_block: None,
            house_bank: None,
            special_gl_indicator: None,
            reference_document_number: None,
            reference_fiscal_year: None,
            reference_line_item: None,
            reference_document_type: None,
            transaction_type: None,
            reference_transaction_type: None,
            reference_key_1: None,
            reference_key_2: None,
            reference_key_3: None,
            financial_area: None,
            consolidation_unit: None,
            partner_company: None,
            trading_partner: None,
            local_currency,
            group_currency: None,
            global_currency: None,
            amount_in_object_currency: None,
            amount_in_profit_center_currency: None,
            dunning_key: None,
            dunning_block: None,
            last_dunning_date: None,
            dunning_level: None,
            discount_days_1: None,
            discount_days_2: None,
            net_payment_days: None,
            discount_percent_1: None,
            discount_percent_2: None,
            discount_amount: None,
            sending_cost_center: None,
            partner_profit_center: None,
            sending_financial_area: None,
            account_assignment: None,
            local_account: None,
            data_source: None,
            split_method: None,
            manual_split: false,
            created_by,
            created_at: chrono::Utc::now().naive_utc(),
            changed_by: None,
            changed_at: None,
            source_module,
            extension_fields: HashMap::new(),
        }
    }

    /// 判断是否为未清项
    pub fn is_open_item(&self) -> bool {
        self.clearing_document.is_none()
    }

    /// 判断是否为已清项
    pub fn is_cleared_item(&self) -> bool {
        self.clearing_document.is_some()
    }

    /// 获取主键
    pub fn get_primary_key(&self) -> String {
        format!(
            "{}-{}-{}-{}-{}",
            self.ledger,
            self.company_code,
            self.fiscal_year,
            self.document_number,
            self.document_line
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_create_universal_journal_entry() {
        let entry = UniversalJournalEntry::new(
            "0L".to_string(),
            "1000".to_string(),
            2026,
            "1000000001".to_string(),
            1,
            "SA".to_string(),
            NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            1,
            "CNY".to_string(),
            "CNY".to_string(),
            "40".to_string(),
            "S".to_string(),
            AccountType::GL,
            "1001".to_string(),
            dec!(10000.00),
            dec!(10000.00),
            SourceModule::GL,
            "USER001".to_string(),
        );

        assert_eq!(entry.ledger, "0L");
        assert_eq!(entry.company_code, "1000");
        assert_eq!(entry.fiscal_year, 2026);
        assert_eq!(entry.document_number, "1000000001");
        assert_eq!(entry.document_line, 1);
        assert_eq!(entry.amount_in_document_currency, dec!(10000.00));
        assert!(entry.is_open_item());
        assert!(!entry.is_cleared_item());
    }

    #[test]
    fn test_get_primary_key() {
        let entry = UniversalJournalEntry::new(
            "0L".to_string(),
            "1000".to_string(),
            2026,
            "1000000001".to_string(),
            1,
            "SA".to_string(),
            NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            1,
            "CNY".to_string(),
            "CNY".to_string(),
            "40".to_string(),
            "S".to_string(),
            AccountType::GL,
            "1001".to_string(),
            dec!(10000.00),
            dec!(10000.00),
            SourceModule::GL,
            "USER001".to_string(),
        );

        assert_eq!(entry.get_primary_key(), "0L-1000-2026-1000000001-1");
    }
}
