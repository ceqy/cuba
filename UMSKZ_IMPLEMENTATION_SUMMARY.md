# UMSKZ - 特殊总账标识 (Special GL Indicator) 实施总结

## 📋 概述

成功实现了 SAP UMSKZ (Special G/L Indicator) 字段，用于区分特殊业务类型（票据、预付款、预收款等）。

**实施日期**: 2026-01-18
**影响范围**: GL Service (总账服务)、Cuba Finance 库、数据库架构

---

## 🎯 业务价值

### SAP 字段说明
- **字段名**: UMSKZ (Special G/L Indicator)
- **用途**: 区分特殊业务类型
- **常见值**:
  - `空值` = 普通业务
  - `A` = 票据 (Bills of Exchange)
  - `F` = 预付款 (Down Payment)
  - `V` = 预收款 (Advance Payment)
  - `W` = 票据贴现 (Bill of Exchange Discount)

### 业务影响
✅ 支持应收/应付账款的分类管理
✅ 提高财务报表的准确性（预付款需要单独列示）
✅ 支持特殊总账项目的清账规则
✅ 符合 SAP S/4HANA Universal Journal 架构

---

## 📁 文件变更清单

### 1. Proto 定义 (API 接口层)

**文件**: `protos/fi/gl/gl.proto`

#### 变更内容:
1. **添加枚举定义** (行 381-391):
```protobuf
// 特殊总账类型 (Special G/L Type / UMSKZ)
enum SpecialGlType {
  SPECIAL_GL_TYPE_UNSPECIFIED = 0;
  SPECIAL_GL_TYPE_NORMAL = 1;           // 普通业务
  SPECIAL_GL_TYPE_BILL_OF_EXCHANGE = 2; // A - 票据
  SPECIAL_GL_TYPE_DOWN_PAYMENT = 3;     // F - 预付款
  SPECIAL_GL_TYPE_ADVANCE_PAYMENT = 4;  // V - 预收款
  SPECIAL_GL_TYPE_BILL_DISCOUNT = 5;    // W - 票据贴现
}
```

2. **更新 JournalEntryLineItem 消息** (行 438-456):
```protobuf
message JournalEntryLineItem {
  // ... 现有字段 ...

  // 特殊总账标识 (UMSKZ)
  string special_gl_indicator = 22;    // 特殊总账标识

  // 并行会计字段
  string ledger = 50;
  LedgerType ledger_type = 51;
  common.v1.MonetaryValue amount_in_ledger_currency = 52;
}
```

---

### 2. 数据库迁移

**文件**: `apps/fi/gl-service/migrations/20260118000001_add_special_gl_indicator.sql`

#### 变更内容:
1. **添加字段到凭证行表**:
```sql
ALTER TABLE journal_entry_lines
ADD COLUMN IF NOT EXISTS special_gl_indicator VARCHAR(1) DEFAULT '';
```

2. **创建索引** (性能优化):
```sql
-- 按特殊总账标识查询的索引
CREATE INDEX IF NOT EXISTS idx_journal_entry_lines_special_gl
ON journal_entry_lines(special_gl_indicator)
WHERE special_gl_indicator IS NOT NULL AND special_gl_indicator != '';

-- 复合索引：支持按科目、特殊总账标识查询
CREATE INDEX IF NOT EXISTS idx_journal_lines_account_special_gl
ON journal_entry_lines(account_id, special_gl_indicator)
WHERE special_gl_indicator IS NOT NULL AND special_gl_indicator != '';
```

3. **添加约束** (数据完整性):
```sql
ALTER TABLE journal_entry_lines
ADD CONSTRAINT chk_special_gl_indicator
CHECK (
    special_gl_indicator = '' OR
    special_gl_indicator IN ('A', 'F', 'V', 'W')
);
```

4. **创建业务视图**:
   - `v_special_gl_items`: 特殊总账项目明细视图
   - `v_special_gl_summary`: 特殊总账汇总视图（按类型、科目、期间汇总）

---

### 3. 领域模型 (Domain Layer)

**文件**: `apps/fi/gl-service/src/domain/aggregates/journal_entry.rs`

#### 变更内容:

1. **添加 SpecialGlType 枚举** (行 100-157):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialGlType {
    Normal,           // 普通业务
    BillOfExchange,   // A - 票据
    DownPayment,      // F - 预付款
    AdvancePayment,   // V - 预收款
    BillDiscount,     // W - 票据贴现
}

impl SpecialGlType {
    pub fn to_sap_code(&self) -> &str { /* ... */ }
    pub fn from_sap_code(code: &str) -> Self { /* ... */ }
    pub fn description(&self) -> &str { /* ... */ }
}
```

2. **更新 LineItem 结构体** (行 158-175):
```rust
pub struct LineItem {
    // ... 现有字段 ...
    pub special_gl_indicator: SpecialGlType,   // 特殊总账类型
    // ... 并行会计字段 ...
}
```

3. **更新构造函数**:
   - `LineItem::new()`: 默认使用 `SpecialGlType::Normal`
   - `LineItem::with_ledger()`: 支持并行会计
   - `LineItem::with_special_gl()`: 新增，用于创建特殊总账行项目

4. **更新冲销逻辑** (行 318-357):
   - 冲销凭证时保留原凭证的 `special_gl_indicator`

---

### 4. 持久化层 (Infrastructure Layer)

**文件**: `apps/fi/gl-service/src/infrastructure/persistence/postgres_journal_repository.rs`

#### 变更内容:

1. **更新 INSERT 语句** (行 68-96):
```rust
INSERT INTO journal_entry_lines (
    id, journal_entry_id, line_number, account_id,
    debit_credit, amount, local_amount,
    cost_center, profit_center, line_text,
    special_gl_indicator, ledger, ledger_type, ledger_amount
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
```

2. **更新 SELECT 语句** (行 117-149):
```rust
SELECT
    id, line_number, account_id, debit_credit, amount, local_amount,
    cost_center, profit_center, line_text,
    special_gl_indicator, ledger, ledger_type, ledger_amount
FROM journal_entry_lines
WHERE journal_entry_id = $1
ORDER BY line_number ASC
```

3. **添加字段映射逻辑**:
```rust
let special_gl_code: String = l.get::<Option<String>, _>("special_gl_indicator")
    .unwrap_or_default();
let special_gl_indicator = SpecialGlType::from_sap_code(&special_gl_code);
```

---

### 5. 应用层 (Application Layer)

**文件**: `apps/fi/gl-service/src/application/commands.rs`

#### 变更内容:
```rust
pub struct LineItemDTO {
    pub account_id: String,
    pub debit_credit: String,
    pub amount: Decimal,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub text: Option<String>,
    pub special_gl_indicator: Option<String>, // 新增: UMSKZ
    pub ledger: Option<String>,
    pub ledger_type: Option<i32>,
    pub ledger_amount: Option<Decimal>,
}
```

**文件**: `apps/fi/gl-service/src/application/handlers.rs`

#### 变更内容:
- `CreateJournalEntryHandler`: 解析 `special_gl_indicator` 字段
- `UpdateJournalEntryHandler`: 支持更新 `special_gl_indicator`

---

### 6. API 层 (gRPC Server)

**文件**: `apps/fi/gl-service/src/api/grpc_server.rs`

#### 变更内容:

1. **请求映射** (行 80-103):
```rust
let lines: Result<Vec<LineItemDTO>, Status> = req.line_items.into_iter().map(|l| {
    // ...
    Ok(LineItemDTO {
        // ... 现有字段 ...
        special_gl_indicator: if l.special_gl_indicator.is_empty() {
            None
        } else {
            Some(l.special_gl_indicator)
        },
        // ...
    })
}).collect();
```

2. **响应映射** (行 1041-1076):
```rust
line_items: entry.lines.into_iter().map(|l| JournalEntryLineItem {
    // ... 现有字段 ...
    special_gl_indicator: l.special_gl_indicator.to_sap_code().to_string(),
    // ...
}).collect(),
```

---

### 7. 共享库 (Cuba Finance)

**文件**: `libs/cuba-finance/src/gl_client.rs`

#### 变更内容:

1. **更新 GlLineItem 结构体** (行 159-172):
```rust
pub struct GlLineItem {
    pub gl_account: String,
    pub debit_credit: String,
    pub amount: rust_decimal::Decimal,
    pub cost_center: Option<String>,
    pub profit_center: Option<String>,
    pub item_text: Option<String>,
    pub business_partner: Option<String>,
    pub special_gl_indicator: Option<String>, // 新增: UMSKZ
    pub ledger: Option<String>,
    pub ledger_type: Option<i32>,
}
```

2. **更新 GL 客户端映射** (行 99-144):
```rust
gl_v1::JournalEntryLineItem {
    // ... 现有字段 ...
    special_gl_indicator: item.special_gl_indicator.unwrap_or_default(),
    // ...
}
```

---

## 🔍 数据库视图说明

### v_special_gl_items (特殊总账项目视图)
用于查询所有特殊总账业务的明细信息。

**字段**:
- 公司代码、凭证号、会计年度、会计期间
- 凭证日期、过账日期
- 科目、业务伙伴
- 特殊总账标识及描述
- 金额、货币
- 清账状态

### v_special_gl_summary (特殊总账汇总视图)
用于报表和分析，按类型、科目、期间汇总。

**字段**:
- 公司代码、会计年度、会计期间
- 特殊总账类型及描述
- 科目、借贷方向
- 交易笔数
- 总金额、未清金额、已清金额

---

## ✅ 测试验证

### 编译验证
```bash
cargo check --package gl-service
# ✅ Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.31s
```

### 功能覆盖
- ✅ Proto 定义完整
- ✅ 数据库迁移脚本完整
- ✅ 领域模型支持完整
- ✅ 持久化层支持完整
- ✅ 应用层支持完整
- ✅ API 层支持完整
- ✅ 共享库支持完整

---

## 📊 使用示例

### 1. 创建预付款凭证 (Down Payment)

```json
{
  "header": {
    "company_code": "1000",
    "fiscal_year": 2026,
    "posting_date": "2026-01-18T00:00:00Z",
    "document_date": "2026-01-18T00:00:00Z",
    "currency": "CNY"
  },
  "line_items": [
    {
      "line_item_number": 1,
      "gl_account": "1100",
      "debit_credit_indicator": "S",
      "amount_in_document_currency": {
        "value": "10000.00",
        "currency_code": "CNY"
      },
      "special_gl_indicator": "F",
      "text": "预付款给供应商"
    },
    {
      "line_item_number": 2,
      "gl_account": "2100",
      "debit_credit_indicator": "H",
      "amount_in_document_currency": {
        "value": "10000.00",
        "currency_code": "CNY"
      },
      "text": "银行存款"
    }
  ]
}
```

### 2. 创建票据凭证 (Bill of Exchange)

```json
{
  "line_items": [
    {
      "gl_account": "1120",
      "debit_credit_indicator": "S",
      "amount_in_document_currency": {
        "value": "50000.00",
        "currency_code": "CNY"
      },
      "special_gl_indicator": "A",
      "text": "应收票据"
    }
  ]
}
```

### 3. 查询特殊总账项目

```sql
-- 查询所有预付款项目
SELECT * FROM v_special_gl_items
WHERE special_gl_indicator = 'F'
  AND clearing_status = 'OPEN';

-- 查询特殊总账汇总
SELECT * FROM v_special_gl_summary
WHERE fiscal_year = 2026
  AND fiscal_period = 1;
```

---

## 🚀 后续工作建议

### 阶段 2: 业务逻辑增强
1. **清账规则**: 实现特殊总账项目的专用清账逻辑
2. **报表功能**: 添加预付款/预收款专用报表
3. **验证规则**: 添加特殊总账标识与科目类型的匹配验证

### 阶段 3: 集成测试
1. **AP/AR 集成**: 确保应付/应收服务正确使用特殊总账标识
2. **报表集成**: 验证财务报表正确列示特殊总账项目
3. **清账测试**: 验证特殊总账项目的清账流程

### 阶段 4: 性能优化
1. **索引优化**: 根据实际查询模式调整索引
2. **视图优化**: 优化视图查询性能
3. **缓存策略**: 考虑添加特殊总账类型的缓存

---

## 📝 注意事项

1. **向后兼容**: 所有现有凭证的 `special_gl_indicator` 默认为空（普通业务）
2. **数据完整性**: 数据库约束确保只能使用有效的特殊总账标识
3. **性能考虑**: 已创建必要的索引，支持高效查询
4. **SAP 兼容**: 字段定义和值完全符合 SAP UMSKZ 规范

---

## 🎉 总结

成功实现了 UMSKZ (Special G/L Indicator) 功能，涵盖：
- ✅ 完整的 API 定义（Proto）
- ✅ 数据库架构变更（迁移脚本 + 视图）
- ✅ 领域模型支持（枚举 + 转换逻辑）
- ✅ 持久化层支持（读写操作）
- ✅ 应用层支持（命令处理）
- ✅ API 层支持（gRPC 映射）
- ✅ 共享库支持（GL Client）

该实现为财务系统提供了完整的特殊总账业务支持，符合 SAP S/4HANA 标准，并为后续的业务扩展奠定了基础。
