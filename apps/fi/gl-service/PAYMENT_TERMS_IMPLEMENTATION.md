# 付款条件详细字段实现总结 (Payment Terms Detail Implementation Summary)

## 概述 (Overview)

本次实现为 GL 服务添加了完整的付款条件详细字段支持，用于精确计算现金折扣和到期日，支持单级和双级折扣场景。

## 实现内容 (Implementation Details)

### 1. Proto 定义 (Proto Definitions)

#### PaymentTermsDetail 消息 (`protos/fi/gl/gl.proto`)

```protobuf
message PaymentTermsDetail {
  google.protobuf.Timestamp baseline_date = 1;  // ZFBDT 现金折扣基准日
  int32 discount_days_1 = 2;                    // ZBD1T 第一个折扣天数
  int32 discount_days_2 = 3;                    // ZBD2T 第二个折扣天数
  int32 net_payment_days = 4;                   // ZBD3T 净付款天数
  string discount_percent_1 = 5;                // ZBD1P 第一个折扣百分比
  string discount_percent_2 = 6;                // ZBD2P 第二个折扣百分比
  common.v1.MonetaryValue discount_amount = 7;  // SKFBT 现金折扣金额
}
```

#### JournalEntryLineItem 字段添加

在 `JournalEntryLineItem` 消息中添加了 `payment_terms_detail` 字段。

### 2. Rust 领域模型 (Rust Domain Model)

#### PaymentTermsDetail 结构体

位置：`apps/fi/gl-service/src/domain/aggregates/journal_entry.rs`

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentTermsDetail {
    pub baseline_date: NaiveDate,              // ZFBDT 现金折扣基准日
    pub discount_days_1: Option<i32>,          // ZBD1T 第一个折扣天数
    pub discount_days_2: Option<i32>,          // ZBD2T 第二个折扣天数
    pub net_payment_days: i32,                 // ZBD3T 净付款天数
    pub discount_percent_1: Option<Decimal>,   // ZBD1P 第一个折扣百分比
    pub discount_percent_2: Option<Decimal>,   // ZBD2P 第二个折扣百分比
    pub discount_amount: Option<Decimal>,      // SKFBT 现金折扣金额
}
```

#### 主要方法

**创建方法：**
- `new(baseline_date, net_payment_days)` - 创建无折扣的付款条件
- `with_single_discount()` - 创建单级折扣（如 2/10 net 30）
- `with_double_discount()` - 创建双级折扣（如 3/10, 2/20 net 30）

**计算方法：**
- `calculate_discount_date_1()` - 计算第一个折扣到期日
- `calculate_discount_date_2()` - 计算第二个折扣到期日
- `calculate_net_due_date()` - 计算净付款到期日
- `calculate_discount_amount()` - 计算现金折扣金额
- `calculate_net_payment_amount()` - 计算净付款金额（扣除折扣后）

**查询方法：**
- `get_applicable_discount_percent()` - 获取适用的折扣百分比
- `has_discount()` - 判断是否有折扣
- `is_within_discount_period()` - 判断付款日期是否在折扣期内
- `get_terms_description()` - 获取付款条件描述（如 "2/10 net 30"）

**验证方法：**
- `validate()` - 验证付款条件的有效性

#### LineItem 字段添加

在 `LineItem` 结构体中添加了 `payment_terms_detail` 字段：

```rust
pub payment_terms_detail: Option<PaymentTermsDetail>, // 付款条件详细信息
```

### 3. SAP 字段映射 (SAP Field Mapping)

| Rust 字段 | SAP 字段 | 描述 |
|----------|---------|------|
| baseline_date | ZFBDT | 现金折扣基准日 |
| discount_days_1 | ZBD1T | 第一个折扣天数 |
| discount_days_2 | ZBD2T | 第二个折扣天数 |
| net_payment_days | ZBD3T | 净付款天数 |
| discount_percent_1 | ZBD1P | 第一个折扣百分比 |
| discount_percent_2 | ZBD2P | 第二个折扣百分比 |
| discount_amount | SKFBT | 现金折扣金额 |

### 4. 使用场景 (Use Cases)

#### 场景 1：单级折扣（2/10 net 30）

```rust
let baseline = NaiveDate::from_ymd_opt(2026, 1, 18).unwrap();
let terms = PaymentTermsDetail::with_single_discount(
    baseline,
    10,        // 10天内付款
    dec!(2.0), // 享受2%折扣
    30,        // 净付款期30天
);

// 计算折扣金额
let invoice_amount = dec!(10000.00);
let payment_date = NaiveDate::from_ymd_opt(2026, 1, 25).unwrap(); // 第7天付款
let discount = terms.calculate_discount_amount(invoice_amount, payment_date);
// discount = 200.00 (10000 * 2%)

let net_amount = terms.calculate_net_payment_amount(invoice_amount, payment_date);
// net_amount = 9800.00 (10000 - 200)
```

#### 场景 2：双级折扣（3/10, 2/20 net 30）

```rust
let baseline = NaiveDate::from_ymd_opt(2026, 1, 18).unwrap();
let terms = PaymentTermsDetail::with_double_discount(
    baseline,
    10,        // 10天内付款享受3%折扣
    dec!(3.0),
    20,        // 20天内付款享受2%折扣
    dec!(2.0),
    30,        // 净付款期30天
);

let invoice_amount = dec!(10000.00);

// 第5天付款 - 享受3%折扣
let payment_date_1 = NaiveDate::from_ymd_opt(2026, 1, 23).unwrap();
let discount_1 = terms.calculate_discount_amount(invoice_amount, payment_date_1);
// discount_1 = 300.00 (10000 * 3%)

// 第15天付款 - 享受2%折扣
let payment_date_2 = NaiveDate::from_ymd_opt(2026, 2, 2).unwrap();
let discount_2 = terms.calculate_discount_amount(invoice_amount, payment_date_2);
// discount_2 = 200.00 (10000 * 2%)

// 第25天付款 - 无折扣
let payment_date_3 = NaiveDate::from_ymd_opt(2026, 2, 12).unwrap();
let discount_3 = terms.calculate_discount_amount(invoice_amount, payment_date_3);
// discount_3 = 0.00
```

#### 场景 3：应付账款凭证带付款条件

```rust
let baseline = NaiveDate::from_ymd_opt(2026, 1, 18).unwrap();
let payment_terms = PaymentTermsDetail::with_single_discount(
    baseline,
    10,
    dec!(2.0),
    30,
);

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
    ).with_payment_terms(payment_terms),
];

let entry = JournalEntry::new(
    "1000".to_string(),
    2026,
    NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
    NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
    "CNY".to_string(),
    Some("采购发票 - 2/10 net 30".to_string()),
    lines,
    None,
).unwrap();
```

### 5. 现金折扣计算逻辑 (Cash Discount Calculation Logic)

#### 折扣计算规则

1. **基准日（Baseline Date）**：现金折扣计算的起始日期
2. **折扣期（Discount Period）**：从基准日开始计算的天数
3. **折扣百分比（Discount Percent）**：在折扣期内付款可享受的折扣比例

#### 计算公式

```
折扣金额 = 发票金额 × 折扣百分比 / 100
净付款金额 = 发票金额 - 折扣金额
```

#### 折扣期判断逻辑

```rust
pub fn get_applicable_discount_percent(&self, payment_date: NaiveDate) -> Option<Decimal> {
    // 检查第一个折扣期
    if let Some(discount_date_1) = self.calculate_discount_date_1() {
        if payment_date <= discount_date_1 {
            return self.discount_percent_1;
        }
    }

    // 检查第二个折扣期
    if let Some(discount_date_2) = self.calculate_discount_date_2() {
        if payment_date <= discount_date_2 {
            return self.discount_percent_2;
        }
    }

    // 超过所有折扣期，无折扣
    None
}
```

### 6. 业务影响 (Business Impact)

#### 现金流管理
- **提前付款激励**：通过现金折扣鼓励客户提前付款
- **资金周转**：改善公司现金流状况
- **折扣成本控制**：精确计算和控制现金折扣成本

#### 应付账款管理
- **付款计划**：根据折扣期优化付款时间
- **折扣利用**：最大化利用供应商提供的现金折扣
- **到期日管理**：自动计算各级折扣到期日和净付款到期日

#### 财务报表
- **折扣收入/费用**：准确记录现金折扣收入或费用
- **应收/应付账款**：反映实际应收或应付金额
- **现金流预测**：基于付款条件预测现金流

### 7. 测试覆盖 (Test Coverage)

实现了 13 个测试用例，覆盖以下场景：

1. `test_payment_terms_creation` - 创建基本付款条件
2. `test_payment_terms_single_discount` - 单级折扣
3. `test_payment_terms_double_discount` - 双级折扣
4. `test_payment_terms_calculate_dates` - 计算到期日
5. `test_payment_terms_calculate_discount_amount` - 计算折扣金额
6. `test_payment_terms_double_discount_calculation` - 双级折扣计算
7. `test_payment_terms_calculate_net_payment_amount` - 计算净付款金额
8. `test_payment_terms_validation` - 验证付款条件
9. `test_line_item_with_payment_terms` - 行项目带付款条件
10. `test_line_item_builder_with_payment_terms` - 构建器创建
11. `test_accounts_payable_with_payment_terms` - 应付账款凭证
12. `test_payment_terms_is_within_discount_period` - 判断折扣期
13. `test_payment_terms_serialization` - 序列化和反序列化

所有测试均通过 ✅

### 8. 付款条件示例 (Payment Terms Examples)

| 描述 | 格式 | 说明 |
|-----|------|------|
| 无折扣 | net 30 | 30天内付款，无折扣 |
| 单级折扣 | 2/10 net 30 | 10天内付款享受2%折扣，否则30天内全额付款 |
| 双级折扣 | 3/10, 2/20 net 30 | 10天内付款享受3%折扣，20天内付款享受2%折扣，否则30天内全额付款 |
| 即期付款 | net 0 | 立即付款 |
| 月结 | net 60 | 60天内付款 |

### 9. 验证规则 (Validation Rules)

1. **净付款天数**：必须大于 0
2. **折扣天数**：必须大于 0 且小于净付款天数
3. **折扣顺序**：第二个折扣天数必须大于第一个折扣天数
4. **折扣百分比**：必须在 0-100 之间
5. **折扣递减**：第二个折扣百分比必须小于第一个折扣百分比

### 10. 文件修改清单 (Modified Files)

1. `apps/fi/gl-service/src/domain/aggregates/journal_entry.rs`
   - 添加 `PaymentTermsDetail` 结构体和实现
   - 添加 `payment_terms_detail` 字段到 `LineItem`
   - 添加 `with_payment_terms()` 方法
   - 更新 `LineItemBuilder`
   - 添加 13 个测试用例

2. `apps/fi/gl-service/src/application/handlers.rs`
   - 更新处理器以支持新字段

3. `apps/fi/gl-service/src/infrastructure/persistence/postgres_journal_repository.rs`
   - 更新数据库持久化代码

## 后续工作 (Future Work)

1. **数据库迁移脚本** - 添加 payment_terms_detail 相关字段到数据库表
2. **gRPC 映射完善** - 完善 proto 到 domain 模型的双向映射
3. **自动折扣计算** - 在付款时自动计算和应用现金折扣
4. **付款建议功能** - 根据付款条件生成最优付款建议
5. **折扣报表** - 生成现金折扣利用率报表

## 总结 (Summary)

本次实现完整地添加了付款条件详细字段支持，包括：
- ✅ Rust 领域模型（完整的 PaymentTermsDetail 结构体）
- ✅ 现金折扣计算逻辑（单级和双级折扣）
- ✅ 到期日计算（折扣到期日和净付款到期日）
- ✅ 构建器模式支持
- ✅ 验证逻辑（完整的业务规则验证）
- ✅ 完整的测试覆盖（13个测试用例）
- ✅ 中文注释和文档

所有功能均已测试通过，可以支持应付账款和应收账款的付款条件管理和现金折扣计算。

## 业务价值 (Business Value)

1. **提高现金流**：通过现金折扣激励提前付款
2. **降低成本**：最大化利用供应商折扣
3. **自动化计算**：减少人工计算错误
4. **精确管理**：精确到天的折扣期管理
5. **灵活配置**：支持单级和双级折扣场景
