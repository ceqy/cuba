# UMSKZ 快速参考指南

## 🎯 什么是 UMSKZ?

UMSKZ (Special G/L Indicator) 是 SAP 中用于标识特殊总账业务类型的字段。

## 📋 有效值

| 代码 | 类型 | 说明 | 使用场景 |
|------|------|------|----------|
| (空) | Normal | 普通业务 | 常规应收/应付、费用等 |
| A | Bill of Exchange | 票据 | 应收票据、应付票据 |
| F | Down Payment | 预付款 | 采购预付款 |
| V | Advance Payment | 预收款 | 销售预收款 |
| W | Bill Discount | 票据贴现 | 票据贴现业务 |

## 💻 代码示例

### Rust (Domain Model)

```rust
use crate::domain::aggregates::journal_entry::{LineItem, SpecialGlType};

// 创建普通业务行项目
let normal_line = LineItem::new(
    1,
    "1100".to_string(),
    DebitCredit::Debit,
    dec!(1000.00),
    dec!(1000.00),
);

// 创建预付款行项目
let down_payment_line = LineItem::with_special_gl(
    2,
    "1100".to_string(),
    DebitCredit::Debit,
    dec!(5000.00),
    dec!(5000.00),
    SpecialGlType::DownPayment,
);

// 转换为 SAP 代码
let sap_code = down_payment_line.special_gl_indicator.to_sap_code(); // "F"

// 从 SAP 代码转换
let special_gl = SpecialGlType::from_sap_code("A"); // BillOfExchange
```

### gRPC (Proto)

```protobuf
message JournalEntryLineItem {
  int32 line_item_number = 1;
  string gl_account = 4;
  string debit_credit_indicator = 3;
  common.v1.MonetaryValue amount_in_document_currency = 8;

  // 特殊总账标识
  string special_gl_indicator = 22;  // "A", "F", "V", "W", or ""
}
```

### GL Client (Cuba Finance)

```rust
use cuba_finance::gl_client::{GlClient, GlLineItem};
use rust_decimal_macros::dec;

// 创建预付款凭证
let line_items = vec![
    GlLineItem {
        gl_account: "1100".to_string(),
        debit_credit: "S".to_string(),
        amount: dec!(10000.00),
        cost_center: None,
        profit_center: None,
        item_text: Some("预付款给供应商".to_string()),
        business_partner: Some("VENDOR001".to_string()),
        special_gl_indicator: Some("F".to_string()), // 预付款
        ledger: None,
        ledger_type: None,
    },
    GlLineItem {
        gl_account: "2100".to_string(),
        debit_credit: "H".to_string(),
        amount: dec!(10000.00),
        cost_center: None,
        profit_center: None,
        item_text: Some("银行存款".to_string()),
        business_partner: None,
        special_gl_indicator: None, // 普通业务
        ledger: None,
        ledger_type: None,
    },
];

let response = gl_client.create_invoice_journal_entry(
    "1000",
    document_date,
    posting_date,
    2026,
    "CNY",
    Some("PO-12345".to_string()),
    Some("预付款凭证".to_string()),
    line_items,
    None,
).await?;
```

## 🗄️ 数据库查询

### 查询所有预付款项目

```sql
SELECT
    company_code,
    document_number,
    fiscal_year,
    account_id,
    amount,
    local_amount,
    clearing_status
FROM v_special_gl_items
WHERE special_gl_indicator = 'F'
  AND clearing_status = 'OPEN'
ORDER BY posting_date DESC;
```

### 查询特殊总账汇总

```sql
SELECT
    fiscal_year,
    fiscal_period,
    special_gl_description,
    transaction_count,
    total_local_amount,
    open_amount,
    cleared_amount
FROM v_special_gl_summary
WHERE fiscal_year = 2026
  AND fiscal_period = 1
ORDER BY special_gl_indicator, account_id;
```

### 按类型统计

```sql
SELECT
    special_gl_indicator,
    CASE special_gl_indicator
        WHEN 'A' THEN '票据'
        WHEN 'F' THEN '预付款'
        WHEN 'V' THEN '预收款'
        WHEN 'W' THEN '票据贴现'
        ELSE '普通业务'
    END as type_name,
    COUNT(*) as count,
    SUM(local_amount) as total_amount
FROM journal_entry_lines
WHERE special_gl_indicator IS NOT NULL
  AND special_gl_indicator != ''
GROUP BY special_gl_indicator;
```

## 🔧 常见问题

### Q1: 如何判断一个凭证行是否为特殊总账项目?

```rust
if line.special_gl_indicator != SpecialGlType::Normal {
    println!("这是特殊总账项目: {}", line.special_gl_indicator.description());
}
```

### Q2: 如何在 AP/AR 服务中创建预付款凭证?

```rust
// 在 AP Service 中
let gl_line_items = vec![
    GlLineItem {
        gl_account: "1100".to_string(),
        debit_credit: "S".to_string(),
        amount: invoice_amount,
        special_gl_indicator: Some("F".to_string()), // 标记为预付款
        // ... 其他字段
    },
    // ... 其他行项目
];

gl_client.create_invoice_journal_entry(
    company_code,
    document_date,
    posting_date,
    fiscal_year,
    currency,
    reference,
    header_text,
    gl_line_items,
    None,
).await?;
```

### Q3: 如何查询某个供应商的所有预付款?

```sql
SELECT
    je.document_number,
    je.posting_date,
    jel.amount,
    jel.local_amount,
    jel.clearing_document,
    CASE
        WHEN jel.clearing_document IS NOT NULL THEN '已清账'
        ELSE '未清账'
    END as status
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE jel.special_gl_indicator = 'F'
  AND jel.business_partner = 'VENDOR001'
ORDER BY je.posting_date DESC;
```

## 📊 报表示例

### 预付款余额表

```sql
SELECT
    jel.business_partner as vendor_code,
    COUNT(*) as transaction_count,
    SUM(CASE WHEN jel.clearing_document IS NULL THEN jel.local_amount ELSE 0 END) as open_balance,
    SUM(CASE WHEN jel.clearing_document IS NOT NULL THEN jel.local_amount ELSE 0 END) as cleared_amount,
    SUM(jel.local_amount) as total_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.special_gl_indicator = 'F'
  AND je.fiscal_year = 2026
GROUP BY jel.business_partner
HAVING SUM(CASE WHEN jel.clearing_document IS NULL THEN jel.local_amount ELSE 0 END) > 0
ORDER BY open_balance DESC;
```

### 票据到期分析

```sql
SELECT
    je.document_number,
    je.posting_date,
    jel.amount,
    jel.clearing_date,
    CASE
        WHEN jel.clearing_date IS NULL THEN '未到期'
        WHEN jel.clearing_date < CURRENT_DATE THEN '已到期'
        ELSE '已清账'
    END as status
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE jel.special_gl_indicator = 'A'
  AND je.status = 'POSTED'
ORDER BY je.posting_date;
```

## ⚠️ 注意事项

1. **数据验证**: 数据库约束确保只能使用有效值 (A, F, V, W, 或空)
2. **向后兼容**: 现有凭证默认为普通业务（空值）
3. **清账规则**: 特殊总账项目可能有特殊的清账规则
4. **报表列示**: 预付款/预收款需要在资产负债表中单独列示

## 🔗 相关文档

- [完整实施总结](./UMSKZ_IMPLEMENTATION_SUMMARY.md)
- [数据库迁移脚本](./apps/fi/gl-service/migrations/20260118000001_add_special_gl_indicator.sql)
- [Proto 定义](./protos/fi/gl/gl.proto)
- [领域模型](./apps/fi/gl-service/src/domain/aggregates/journal_entry.rs)
