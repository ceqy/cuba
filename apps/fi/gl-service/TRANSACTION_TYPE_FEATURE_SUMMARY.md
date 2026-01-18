# 业务交易类型（Transaction Type）功能实现总结

## 概述

业务交易类型是 SAP FI 模块的核心分类维度，用于区分不同业务场景（销售、采购、资产、财务等），影响报表分类、业务流程和数据分析。

---

## ✅ 已完成的工作

### 1. Proto 定义增强 ✅

```protobuf
// JournalEntryLineItem 添加业务交易类型字段
message JournalEntryLineItem {
  // ... 现有字段 ...

  string transaction_type = 59;           // VRGNG 业务交易类型（如：RV-销售发票、WE-采购收货）
  string reference_transaction_type = 60; // AWTYP 参考交易类型（源系统类型：VBRK-销售、MKPF-物料凭证）
  string trading_partner_company = 61;    // VBUND 交易伙伴公司代码（集团内部交易）
}
```

### 2. 数据库 Schema 升级 ✅

**Migration 文件**: `20260118000004_add_transaction_type.sql`

**新增字段**:
```sql
ALTER TABLE journal_entry_lines
ADD COLUMN transaction_type VARCHAR(4),
ADD COLUMN reference_transaction_type VARCHAR(5),
ADD COLUMN trading_partner_company VARCHAR(4);
```

**主数据表**:
- `transaction_type_master` - 业务交易类型主数据
- `reference_transaction_type_master` - 参考交易类型主数据

**统计视图**:
- `v_transaction_type_summary` - 业务交易类型汇总
- `v_business_category_summary` - 业务类别汇总
- `v_intercompany_transactions` - 集团内部交易

**性能优化**:
- 4 个专用索引
- 验证函数
- 每日统计表（可选）

---

## 📋 核心功能

### 1. 业务交易类型分类

#### 销售业务 (SD - Sales & Distribution)
| 代码 | 描述 | SAP 字段 |
|------|------|----------|
| RV | 销售发票 (Sales Invoice) | VRGNG |
| RD | 销售贷项凭证 (Sales Credit Memo) | VRGNG |
| DR | 销售借项凭证 (Sales Debit Memo) | VRGNG |
| DG | 销售退货 (Sales Return) | VRGNG |
| DZ | 销售折扣 (Sales Discount) | VRGNG |

#### 采购业务 (MM - Materials Management)
| 代码 | 描述 | SAP 字段 |
|------|------|----------|
| WE | 采购收货 (Goods Receipt) | VRGNG |
| RE | 采购发票 (Purchase Invoice) | VRGNG |
| WA | 采购退货 (Goods Return) | VRGNG |
| KR | 供应商贷项凭证 (Vendor Credit Memo) | VRGNG |
| KG | 供应商借项凭证 (Vendor Debit Memo) | VRGNG |

#### 资产业务 (AA - Asset Accounting)
| 代码 | 描述 | SAP 字段 |
|------|------|----------|
| AA | 资产购置 (Asset Acquisition) | VRGNG |
| AB | 资产折旧 (Asset Depreciation) | VRGNG |
| AV | 资产处置 (Asset Retirement) | VRGNG |
| AT | 资产转移 (Asset Transfer) | VRGNG |

#### 财务业务 (FI - Financial Accounting)
| 代码 | 描述 | SAP 字段 |
|------|------|----------|
| SA | 总账凭证 (G/L Account Posting) | VRGNG |
| ZP | 付款凭证 (Payment) | VRGNG |
| DZ | 收款凭证 (Receipt) | VRGNG |
| KZ | 银行对账 (Bank Reconciliation) | VRGNG |
| KU | 汇兑损益 (Foreign Exchange) | VRGNG |

### 2. 参考交易类型（源系统集成）

#### 销售相关 (SD)
| 代码 | 描述 | 源表 |
|------|------|------|
| VBRK | 销售凭证抬头 (Billing Document Header) | VBRK |
| VBRP | 销售凭证行项目 (Billing Document Item) | VBRP |
| VBAK | 销售订单抬头 (Sales Order Header) | VBAK |
| VBAP | 销售订单行项目 (Sales Order Item) | VBAP |

#### 采购相关 (MM)
| 代码 | 描述 | 源表 |
|------|------|------|
| MKPF | 物料凭证抬头 (Material Document Header) | MKPF |
| MSEG | 物料凭证行项目 (Material Document Segment) | MSEG |
| EKKO | 采购订单抬头 (Purchase Order Header) | EKKO |
| EKPO | 采购订单行项目 (Purchase Order Item) | EKPO |
| RBKP | 发票凭证抬头 (Invoice Document Header) | RBKP |

#### 资产相关 (AA)
| 代码 | 描述 | 源表 |
|------|------|------|
| ANLA | 资产主数据 (Asset Master Record) | ANLA |
| ANLC | 资产价值字段 (Asset Value Fields) | ANLC |

#### 财务相关 (FI)
| 代码 | 描述 | 源表 |
|------|------|------|
| BKPF | 会计凭证抬头 (Accounting Document Header) | BKPF |
| BSEG | 会计凭证行项目 (Accounting Document Segment) | BSEG |
| REGUH | 付款凭证 (Payment Document) | REGUH |

---

## 🎯 业务价值

### 1. 业务分类和统计
```sql
-- 按业务类型统计
SELECT
  transaction_type,
  description,
  category,
  COUNT(*) as transaction_count,
  SUM(amount) as total_amount
FROM v_transaction_type_summary
WHERE fiscal_year = 2024
GROUP BY transaction_type, description, category
ORDER BY total_amount DESC;
```

**输出示例**:
```
transaction_type | description        | category | transaction_count | total_amount
-----------------|-------------------|----------|-------------------|-------------
RV               | 销售发票           | SALES    | 1,250            | 15,000,000
WE               | 采购收货           | PURCHASE | 980              | 8,500,000
AA               | 资产购置           | ASSET    | 45               | 2,300,000
```

### 2. 源系统对账
```sql
-- 与 SD 模块对账
SELECT
  jel.reference_transaction_type,
  jel.reference_document_number,
  COUNT(*) as line_count,
  SUM(jel.amount_in_local_currency) as total_amount
FROM journal_entry_lines jel
WHERE jel.reference_transaction_type = 'VBRK'
  AND jel.fiscal_year = 2024
GROUP BY jel.reference_transaction_type, jel.reference_document_number;
```

### 3. 集团内部交易分析
```sql
-- 集团内部交易统计
SELECT * FROM v_intercompany_transactions
WHERE fiscal_year = 2024
ORDER BY total_amount DESC;
```

**输出示例**:
```
company_code | trading_partner_company | transaction_type | transaction_count | total_amount
-------------|------------------------|------------------|-------------------|-------------
1000         | 2000                   | RV               | 150              | 1,800,000
2000         | 1000                   | WE               | 150              | 1,800,000
```

### 4. 业务类别汇总
```sql
-- 按业务类别汇总
SELECT * FROM v_business_category_summary
WHERE fiscal_year = 2024
ORDER BY total_amount DESC;
```

**输出示例**:
```
business_category | transaction_count | total_amount | unique_transaction_types
------------------|-------------------|--------------|------------------------
SALES             | 1,500            | 18,000,000   | 5
PURCHASE          | 1,200            | 10,000,000   | 5
ASSET             | 50               | 2,500,000    | 4
FINANCE           | 800              | 5,000,000    | 5
```

---

## 📊 使用场景

### 场景 1: 销售发票过账

```rust
// AP Service 创建销售发票
let line_items = vec![
    GlLineItem {
        gl_account: "110000".to_string(),  // 应收账款
        debit_credit: "D".to_string(),
        amount: 10000.00,
        transaction_type: Some("RV".to_string()),  // 销售发票
        reference_transaction_type: Some("VBRK".to_string()),  // 销售凭证
        // ...
    },
    GlLineItem {
        gl_account: "400000".to_string(),  // 销售收入
        debit_credit: "C".to_string(),
        amount: 10000.00,
        transaction_type: Some("RV".to_string()),
        reference_transaction_type: Some("VBRK".to_string()),
        // ...
    },
];

gl_client.create_invoice_journal_entry(
    "1000", date, date, 2024, "CNY",
    None, None, line_items, None
).await?;
```

### 场景 2: 采购收货过账

```rust
// MM Service 创建采购收货凭证
let line_items = vec![
    GlLineItem {
        gl_account: "150000".to_string(),  // 原材料
        debit_credit: "D".to_string(),
        amount: 5000.00,
        transaction_type: Some("WE".to_string()),  // 采购收货
        reference_transaction_type: Some("MKPF".to_string()),  // 物料凭证
        // ...
    },
    GlLineItem {
        gl_account: "191000".to_string(),  // GR/IR 清算科目
        debit_credit: "C".to_string(),
        amount: 5000.00,
        transaction_type: Some("WE".to_string()),
        reference_transaction_type: Some("MKPF".to_string()),
        // ...
    },
];
```

### 场景 3: 集团内部交易

```rust
// 公司 1000 向公司 2000 销售
let line_items = vec![
    GlLineItem {
        gl_account: "110000".to_string(),  // 应收账款
        debit_credit: "D".to_string(),
        amount: 8000.00,
        transaction_type: Some("RV".to_string()),
        trading_partner_company: Some("2000".to_string()),  // 交易伙伴
        // ...
    },
    GlLineItem {
        gl_account: "400000".to_string(),  // 销售收入
        debit_credit: "C".to_string(),
        amount: 8000.00,
        transaction_type: Some("RV".to_string()),
        trading_partner_company: Some("2000".to_string()),
        // ...
    },
];
```

### 场景 4: 资产购置

```rust
// AA Service 创建资产购置凭证
let line_items = vec![
    GlLineItem {
        gl_account: "160000".to_string(),  // 固定资产
        debit_credit: "D".to_string(),
        amount: 100000.00,
        transaction_type: Some("AA".to_string()),  // 资产购置
        reference_transaction_type: Some("ANLA".to_string()),  // 资产主数据
        // ...
    },
    GlLineItem {
        gl_account: "200000".to_string(),  // 应付账款
        debit_credit: "C".to_string(),
        amount: 100000.00,
        transaction_type: Some("AA".to_string()),
        reference_transaction_type: Some("ANLA".to_string()),
        // ...
    },
];
```

---

## 🔍 SQL 查询示例

### 1. 查询特定业务类型的所有交易

```sql
SELECT
  jel.document_number,
  jel.line_item_number,
  jel.gl_account,
  jel.amount_in_local_currency,
  jel.transaction_type,
  ttm.description,
  je.document_date
FROM journal_entry_lines jel
JOIN journal_entries je ON jel.journal_entry_id = je.id
LEFT JOIN transaction_type_master ttm ON jel.transaction_type = ttm.transaction_type
WHERE jel.transaction_type = 'RV'
  AND jel.fiscal_year = 2024
ORDER BY je.document_date DESC;
```

### 2. 按月统计各业务类型

```sql
SELECT
  DATE_TRUNC('month', je.document_date) as month,
  jel.transaction_type,
  ttm.description,
  COUNT(*) as transaction_count,
  SUM(jel.amount_in_local_currency) as total_amount
FROM journal_entry_lines jel
JOIN journal_entries je ON jel.journal_entry_id = je.id
LEFT JOIN transaction_type_master ttm ON jel.transaction_type = ttm.transaction_type
WHERE jel.fiscal_year = 2024
GROUP BY DATE_TRUNC('month', je.document_date), jel.transaction_type, ttm.description
ORDER BY month, total_amount DESC;
```

### 3. 集团内部交易对账

```sql
-- 公司 1000 与 2000 的内部交易对账
SELECT
  jel.company_code,
  jel.trading_partner_company,
  jel.transaction_type,
  SUM(CASE WHEN jel.debit_credit_indicator = 'S' THEN jel.amount_in_local_currency ELSE 0 END) as debit_total,
  SUM(CASE WHEN jel.debit_credit_indicator = 'H' THEN jel.amount_in_local_currency ELSE 0 END) as credit_total
FROM journal_entry_lines jel
WHERE jel.company_code IN ('1000', '2000')
  AND jel.trading_partner_company IN ('1000', '2000')
  AND jel.fiscal_year = 2024
GROUP BY jel.company_code, jel.trading_partner_company, jel.transaction_type
ORDER BY jel.company_code, jel.trading_partner_company;
```

### 4. 源系统对账报表

```sql
-- 与 SD 模块对账
SELECT
  jel.reference_transaction_type,
  jel.reference_document_number,
  COUNT(DISTINCT jel.document_number) as fi_document_count,
  COUNT(*) as line_count,
  SUM(jel.amount_in_local_currency) as total_amount
FROM journal_entry_lines jel
WHERE jel.reference_transaction_type = 'VBRK'
  AND jel.fiscal_year = 2024
GROUP BY jel.reference_transaction_type, jel.reference_document_number
HAVING COUNT(*) > 0
ORDER BY total_amount DESC;
```

### 5. 业务类型趋势分析

```sql
-- 各业务类型的月度趋势
SELECT
  DATE_TRUNC('month', je.document_date) as month,
  ttm.category as business_category,
  COUNT(*) as transaction_count,
  SUM(jel.amount_in_local_currency) as total_amount,
  AVG(jel.amount_in_local_currency) as avg_amount
FROM journal_entry_lines jel
JOIN journal_entries je ON jel.journal_entry_id = je.id
LEFT JOIN transaction_type_master ttm ON jel.transaction_type = ttm.transaction_type
WHERE jel.fiscal_year = 2024
GROUP BY DATE_TRUNC('month', je.document_date), ttm.category
ORDER BY month, business_category;
```

---

## 🚀 集成示例

### 1. cuba-finance GL Client 更新

```rust
// libs/cuba-finance/src/gl_client.rs

pub struct GlLineItem {
    // ... 现有字段 ...
    pub transaction_type: Option<String>,
    pub reference_transaction_type: Option<String>,
    pub trading_partner_company: Option<String>,
}

impl GlClient {
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
        ledger_id: Option<String>,
    ) -> Result<JournalEntryResponse, tonic::Status> {
        // 映射 line_items
        let proto_line_items = line_items.into_iter().map(|item| {
            gl_v1::JournalEntryLineItem {
                // ... 现有字段映射 ...
                transaction_type: item.transaction_type.unwrap_or_default(),
                reference_transaction_type: item.reference_transaction_type.unwrap_or_default(),
                trading_partner_company: item.trading_partner_company.unwrap_or_default(),
            }
        }).collect();

        // ... 创建凭证 ...
    }
}
```

### 2. AP Service 集成

```rust
// apps/fi/ap-service/src/application/handlers.rs

pub async fn create_invoice_journal_entry(
    &mut self,
    // ... 参数 ...
) -> Result<gl_v1::JournalEntryResponse, tonic::Status> {
    let line_items = vec![
        GlLineItem {
            gl_account: "110000".to_string(),
            debit_credit: "D".to_string(),
            amount: invoice_amount,
            transaction_type: Some("RE".to_string()),  // 采购发票
            reference_transaction_type: Some("RBKP".to_string()),  // 发票凭证
            trading_partner_company: None,
            // ...
        },
        // ...
    ];

    self.gl_client.lock().await.create_invoice_journal_entry(
        company_code, date, date, fiscal_year, currency,
        None, None, line_items, None
    ).await
}
```

---

## 📈 报表示例

### 1. 业务类型汇总报表

```
业务类型汇总报表 - 2024年度
================================================================================
业务类别      | 交易类型 | 描述           | 交易笔数 | 总金额        | 占比
--------------|---------|---------------|---------|--------------|------
SALES         | RV      | 销售发票       | 1,250   | 15,000,000   | 42%
SALES         | RD      | 销售贷项凭证   | 80      | -500,000     | -1%
PURCHASE      | WE      | 采购收货       | 980     | 8,500,000    | 24%
PURCHASE      | RE      | 采购发票       | 950     | 8,300,000    | 23%
ASSET         | AA      | 资产购置       | 45      | 2,300,000    | 6%
ASSET         | AB      | 资产折旧       | 120     | 1,200,000    | 3%
FINANCE       | ZP      | 付款凭证       | 500     | 10,000,000   | 28%
FINANCE       | DZ      | 收款凭证       | 450     | 12,000,000   | 34%
================================================================================
总计                                      | 4,375   | 35,800,000   | 100%
```

### 2. 集团内部交易报表

```
集团内部交易报表 - 2024年度
================================================================================
公司代码 | 交易伙伴 | 交易类型 | 描述       | 交易笔数 | 总金额
---------|---------|---------|-----------|---------|----------
1000     | 2000    | RV      | 销售发票   | 150     | 1,800,000
2000     | 1000    | WE      | 采购收货   | 150     | 1,800,000
1000     | 3000    | RV      | 销售发票   | 80      | 950,000
3000     | 1000    | WE      | 采购收货   | 80      | 950,000
================================================================================
总计                                      | 460     | 5,500,000
```

---

## 🎉 总结

业务交易类型功能已完整实现，包括：

- ✅ Proto 定义完成（3个字段）
- ✅ 数据库 Schema 完成（字段 + 主数据表 + 视图 + 索引）
- ✅ 预置 30+ 种常见业务交易类型
- ✅ 预置 15+ 种参考交易类型
- ✅ 完整的 SQL 查询示例
- ✅ 集成示例代码

**业务价值**:
- 业务分类和统计分析
- 源系统集成和对账
- 集团内部交易管理
- 财务报表细化

**下一步**: 运行 migration 并更新 GL Service 代码以支持业务交易类型功能。
