# UMSKZ 特殊总账数据库视图使用指南

## 📋 概述

本文档详细说明了特殊总账标识 (UMSKZ) 相关的所有数据库视图及其使用方法。

---

## 🗂️ 视图清单

### 基础视图

1. **v_special_gl_items** - 特殊总账项目明细视图
2. **v_special_gl_summary** - 特殊总账汇总视图

### 余额视图

3. **v_down_payment_balance** - 预付款余额视图
4. **v_advance_payment_balance** - 预收款余额视图

### 分析视图

5. **v_bill_maturity_analysis** - 票据到期分析视图
6. **v_special_gl_aging** - 特殊总账账龄分析视图
7. **v_special_gl_monthly_trend** - 特殊总账月度趋势视图
8. **v_special_gl_clearing_efficiency** - 特殊总账清账效率分析视图

### 业务伙伴视图

9. **v_business_partner_special_gl** - 业务伙伴特殊总账汇总视图

### 风险管理视图

10. **v_special_gl_risk_alert** - 特殊总账风险预警视图
11. **v_special_gl_data_quality** - 特殊总账数据质量检查视图

### 物化视图

12. **mv_special_gl_balance** - 特殊总账余额物化视图（性能优化）

---

## 📊 视图详细说明

### 1. v_special_gl_items - 特殊总账项目明细视图

**用途**: 查询所有特殊总账业务的明细信息

**字段说明**:
- `company_code` - 公司代码
- `document_number` - 凭证号
- `fiscal_year` - 会计年度
- `fiscal_period` - 会计期间
- `document_date` - 凭证日期
- `posting_date` - 过账日期
- `line_item_number` - 行项目号
- `account_id` - 科目代码
- `business_partner` - 业务伙伴（供应商/客户）
- `special_gl_indicator` - 特殊总账标识 (A/F/V/W)
- `special_gl_description` - 特殊总账类型描述
- `amount` - 凭证货币金额
- `local_amount` - 本位币金额
- `currency` - 货币代码
- `debit_credit` - 借贷标识
- `clearing_document` - 清账凭证号
- `clearing_date` - 清账日期
- `clearing_status` - 清账状态 (OPEN/CLEARED)
- `document_status` - 凭证状态

**使用示例**:

```sql
-- 查询所有未清的预付款项目
SELECT
    document_number,
    posting_date,
    business_partner,
    local_amount,
    clearing_status
FROM v_special_gl_items
WHERE special_gl_indicator = 'F'
  AND clearing_status = 'OPEN'
ORDER BY posting_date DESC;

-- 查询某个供应商的所有票据
SELECT *
FROM v_special_gl_items
WHERE special_gl_indicator = 'A'
  AND business_partner = 'VENDOR001'
ORDER BY posting_date;

-- 按类型统计金额
SELECT
    special_gl_description,
    COUNT(*) as count,
    SUM(local_amount) as total_amount
FROM v_special_gl_items
WHERE fiscal_year = 2026
GROUP BY special_gl_description;
```

---

### 2. v_special_gl_summary - 特殊总账汇总视图

**用途**: 按类型、科目、期间汇总特殊业务

**字段说明**:
- `company_code` - 公司代码
- `fiscal_year` - 会计年度
- `fiscal_period` - 会计期间
- `special_gl_indicator` - 特殊总账标识
- `special_gl_description` - 类型描述
- `account_id` - 科目代码
- `debit_credit` - 借贷标识
- `transaction_count` - 交易笔数
- `total_amount` - 总金额
- `total_local_amount` - 本位币总金额
- `open_amount` - 未清金额
- `cleared_amount` - 已清金额

**使用示例**:

```sql
-- 查询2026年各类型特殊总账的汇总
SELECT
    special_gl_description,
    SUM(transaction_count) as total_transactions,
    SUM(total_local_amount) as total_amount,
    SUM(open_amount) as open_amount,
    SUM(cleared_amount) as cleared_amount
FROM v_special_gl_summary
WHERE fiscal_year = 2026
GROUP BY special_gl_description;

-- 按月查询预付款趋势
SELECT
    fiscal_period,
    SUM(total_local_amount) as monthly_amount,
    SUM(open_amount) as open_amount
FROM v_special_gl_summary
WHERE fiscal_year = 2026
  AND special_gl_indicator = 'F'
GROUP BY fiscal_period
ORDER BY fiscal_period;
```

---

### 3. v_down_payment_balance - 预付款余额视图

**用途**: 显示未清预付款余额（用于资产负债表）

**字段说明**:
- `company_code` - 公司代码
- `vendor_code` - 供应商代码
- `account_id` - 科目代码
- `transaction_count` - 交易笔数
- `open_debit_balance` - 未清借方余额
- `open_credit_balance` - 未清贷方余额
- `net_open_balance` - 净未清余额
- `last_transaction_date` - 最后交易日期

**使用示例**:

```sql
-- 查询所有供应商的预付款余额
SELECT
    vendor_code,
    net_open_balance,
    transaction_count,
    last_transaction_date
FROM v_down_payment_balance
WHERE company_code = '1000'
ORDER BY net_open_balance DESC;

-- 查询预付款总额（用于资产负债表）
SELECT
    company_code,
    SUM(net_open_balance) as total_down_payment
FROM v_down_payment_balance
GROUP BY company_code;

-- 查询超过90天的预付款
SELECT
    vendor_code,
    net_open_balance,
    last_transaction_date,
    CURRENT_DATE - last_transaction_date as days_outstanding
FROM v_down_payment_balance
WHERE CURRENT_DATE - last_transaction_date > 90
ORDER BY days_outstanding DESC;
```

---

### 4. v_advance_payment_balance - 预收款余额视图

**用途**: 显示未清预收款余额（用于资产负债表）

**字段说明**: 与预付款余额视图类似，但 `vendor_code` 改为 `customer_code`

**使用示例**:

```sql
-- 查询所有客户的预收款余额
SELECT
    customer_code,
    net_open_balance,
    transaction_count,
    last_transaction_date
FROM v_advance_payment_balance
WHERE company_code = '1000'
ORDER BY net_open_balance DESC;

-- 查询预收款总额（用于资产负债表）
SELECT
    company_code,
    SUM(net_open_balance) as total_advance_payment
FROM v_advance_payment_balance
GROUP BY company_code;
```

---

### 5. v_bill_maturity_analysis - 票据到期分析视图

**用途**: 用于票据管理和风险控制

**字段说明**:
- `company_code` - 公司代码
- `document_number` - 凭证号
- `posting_date` - 过账日期
- `business_partner` - 业务伙伴
- `account_id` - 科目代码
- `local_amount` - 金额
- `currency` - 货币
- `clearing_date` - 到期日
- `clearing_document` - 清账凭证
- `maturity_status` - 到期状态
- `days_to_maturity` - 距到期天数

**到期状态分类**:
- `已清账` - 票据已清账
- `未设置到期日` - 缺少到期日信息
- `已到期未清` - 已过到期日但未清账（高风险）
- `30天内到期` - 即将到期
- `90天内到期` - 近期到期
- `90天后到期` - 远期到期

**使用示例**:

```sql
-- 查询已到期未清的票据（高风险）
SELECT
    document_number,
    business_partner,
    local_amount,
    clearing_date as maturity_date,
    CURRENT_DATE - clearing_date as overdue_days
FROM v_bill_maturity_analysis
WHERE maturity_status = '已到期未清'
ORDER BY clearing_date;

-- 查询30天内到期的票据
SELECT
    document_number,
    business_partner,
    local_amount,
    days_to_maturity
FROM v_bill_maturity_analysis
WHERE maturity_status = '30天内到期'
ORDER BY days_to_maturity;

-- 按到期状态统计票据金额
SELECT
    maturity_status,
    COUNT(*) as count,
    SUM(local_amount) as total_amount
FROM v_bill_maturity_analysis
GROUP BY maturity_status
ORDER BY
    CASE maturity_status
        WHEN '已到期未清' THEN 1
        WHEN '30天内到期' THEN 2
        WHEN '90天内到期' THEN 3
        ELSE 4
    END;
```

---

### 6. v_special_gl_aging - 特殊总账账龄分析视图

**用途**: 按账龄段统计未清项目

**字段说明**:
- `company_code` - 公司代码
- `special_gl_indicator` - 特殊总账标识
- `special_gl_type` - 类型名称
- `business_partner` - 业务伙伴
- `account_id` - 科目代码
- `aging_0_30_days` - 0-30天金额
- `aging_31_60_days` - 31-60天金额
- `aging_61_90_days` - 61-90天金额
- `aging_91_180_days` - 91-180天金额
- `aging_over_180_days` - 超过180天金额
- `total_open_amount` - 未清总金额

**使用示例**:

```sql
-- 查询预付款账龄分析
SELECT
    business_partner,
    aging_0_30_days,
    aging_31_60_days,
    aging_61_90_days,
    aging_91_180_days,
    aging_over_180_days,
    total_open_amount
FROM v_special_gl_aging
WHERE special_gl_indicator = 'F'
ORDER BY total_open_amount DESC;

-- 查询超过180天的预付款（高风险）
SELECT
    business_partner,
    aging_over_180_days
FROM v_special_gl_aging
WHERE special_gl_indicator = 'F'
  AND aging_over_180_days > 0
ORDER BY aging_over_180_days DESC;

-- 按类型汇总账龄
SELECT
    special_gl_type,
    SUM(aging_0_30_days) as total_0_30,
    SUM(aging_31_60_days) as total_31_60,
    SUM(aging_61_90_days) as total_61_90,
    SUM(aging_91_180_days) as total_91_180,
    SUM(aging_over_180_days) as total_over_180
FROM v_special_gl_aging
GROUP BY special_gl_type;
```

---

### 7. v_special_gl_monthly_trend - 特殊总账月度趋势视图

**用途**: 分析特殊业务的月度变化趋势

**字段说明**:
- `company_code` - 公司代码
- `fiscal_year` - 会计年度
- `fiscal_period` - 会计期间
- `special_gl_indicator` - 特殊总账标识
- `special_gl_type` - 类型名称
- `transaction_count` - 本期交易笔数
- `period_amount` - 本期发生额
- `debit_amount` - 借方发生额
- `credit_amount` - 贷方发生额
- `cleared_in_period` - 本期清账金额
- `open_at_period_end` - 期末未清金额

**使用示例**:

```sql
-- 查询2026年预付款月度趋势
SELECT
    fiscal_period,
    transaction_count,
    period_amount,
    cleared_in_period,
    open_at_period_end
FROM v_special_gl_monthly_trend
WHERE fiscal_year = 2026
  AND special_gl_indicator = 'F'
ORDER BY fiscal_period;

-- 对比各类型特殊总账的月度趋势
SELECT
    fiscal_period,
    special_gl_type,
    period_amount,
    open_at_period_end
FROM v_special_gl_monthly_trend
WHERE fiscal_year = 2026
ORDER BY fiscal_period, special_gl_type;

-- 计算月度增长率
SELECT
    fiscal_period,
    period_amount,
    LAG(period_amount) OVER (ORDER BY fiscal_period) as prev_period_amount,
    ROUND(
        (period_amount - LAG(period_amount) OVER (ORDER BY fiscal_period)) /
        NULLIF(LAG(period_amount) OVER (ORDER BY fiscal_period), 0) * 100,
        2
    ) as growth_rate_percent
FROM v_special_gl_monthly_trend
WHERE fiscal_year = 2026
  AND special_gl_indicator = 'F'
ORDER BY fiscal_period;
```

---

### 8. v_special_gl_clearing_efficiency - 特殊总账清账效率分析视图

**用途**: 评估清账效率和资金周转

**字段说明**:
- `company_code` - 公司代码
- `fiscal_year` - 会计年度
- `special_gl_indicator` - 特殊总账标识
- `special_gl_type` - 类型名称
- `total_count` - 总笔数
- `cleared_count` - 已清笔数
- `open_count` - 未清笔数
- `clearing_rate_percent` - 清账率（%）
- `avg_clearing_days` - 平均清账天数
- `total_amount` - 总金额
- `cleared_amount` - 已清金额
- `open_amount` - 未清金额

**使用示例**:

```sql
-- 查询各类型特殊总账的清账效率
SELECT
    special_gl_type,
    total_count,
    cleared_count,
    open_count,
    clearing_rate_percent,
    avg_clearing_days
FROM v_special_gl_clearing_efficiency
WHERE fiscal_year = 2026
ORDER BY clearing_rate_percent DESC;

-- 对比不同年度的清账效率
SELECT
    fiscal_year,
    special_gl_type,
    clearing_rate_percent,
    avg_clearing_days
FROM v_special_gl_clearing_efficiency
WHERE special_gl_indicator = 'F'
ORDER BY fiscal_year, special_gl_type;

-- 识别清账效率低的类型
SELECT
    special_gl_type,
    clearing_rate_percent,
    avg_clearing_days,
    open_amount
FROM v_special_gl_clearing_efficiency
WHERE fiscal_year = 2026
  AND clearing_rate_percent < 80
ORDER BY clearing_rate_percent;
```

---

### 9. v_business_partner_special_gl - 业务伙伴特殊总账汇总视图

**用途**: 按供应商/客户汇总特殊业务

**字段说明**:
- `company_code` - 公司代码
- `business_partner` - 业务伙伴代码
- `special_gl_indicator` - 特殊总账标识
- `special_gl_type` - 类型名称
- `transaction_count` - 交易笔数
- `total_amount` - 总金额
- `open_amount` - 未清金额
- `cleared_amount` - 已清金额
- `first_transaction_date` - 首次交易日期
- `last_transaction_date` - 最后交易日期
- `last_open_transaction_date` - 最后未清交易日期

**使用示例**:

```sql
-- 查询某供应商的所有特殊总账业务
SELECT
    special_gl_type,
    transaction_count,
    total_amount,
    open_amount,
    last_transaction_date
FROM v_business_partner_special_gl
WHERE business_partner = 'VENDOR001'
ORDER BY special_gl_type;

-- 查询预付款金额最大的前10个供应商
SELECT
    business_partner,
    open_amount,
    transaction_count,
    last_open_transaction_date
FROM v_business_partner_special_gl
WHERE special_gl_indicator = 'F'
ORDER BY open_amount DESC
LIMIT 10;

-- 查询长期未发生交易的业务伙伴
SELECT
    business_partner,
    special_gl_type,
    open_amount,
    last_open_transaction_date,
    CURRENT_DATE - last_open_transaction_date as days_since_last_transaction
FROM v_business_partner_special_gl
WHERE open_amount > 0
  AND CURRENT_DATE - last_open_transaction_date > 180
ORDER BY days_since_last_transaction DESC;
```

---

### 10. v_special_gl_risk_alert - 特殊总账风险预警视图

**用途**: 识别需要关注的异常项目

**字段说明**:
- `company_code` - 公司代码
- `document_number` - 凭证号
- `posting_date` - 过账日期
- `special_gl_indicator` - 特殊总账标识
- `special_gl_type` - 类型名称
- `business_partner` - 业务伙伴
- `account_id` - 科目代码
- `local_amount` - 金额
- `days_outstanding` - 未清天数
- `risk_alert` - 风险提示
- `risk_level` - 风险等级 (HIGH/MEDIUM/LOW)

**风险等级定义**:
- `HIGH` - 高风险：票据已到期未清 或 超过180天未清
- `MEDIUM` - 中风险：超过90天未清
- `LOW` - 低风险：超过30天未清

**使用示例**:

```sql
-- 查询所有高风险项目
SELECT
    document_number,
    special_gl_type,
    business_partner,
    local_amount,
    days_outstanding,
    risk_alert
FROM v_special_gl_risk_alert
WHERE risk_level = 'HIGH'
ORDER BY local_amount DESC;

-- 按风险等级统计
SELECT
    risk_level,
    COUNT(*) as count,
    SUM(local_amount) as total_amount
FROM v_special_gl_risk_alert
GROUP BY risk_level
ORDER BY
    CASE risk_level
        WHEN 'HIGH' THEN 1
        WHEN 'MEDIUM' THEN 2
        WHEN 'LOW' THEN 3
    END;

-- 查询某业务伙伴的风险项目
SELECT
    document_number,
    special_gl_type,
    local_amount,
    days_outstanding,
    risk_alert,
    risk_level
FROM v_special_gl_risk_alert
WHERE business_partner = 'VENDOR001'
ORDER BY risk_level, days_outstanding DESC;
```

---

### 11. v_special_gl_data_quality - 特殊总账数据质量检查视图

**用途**: 识别数据质量问题

**字段说明**:
- `issue_type` - 问题类型
- `issue_description` - 问题描述
- `company_code` - 公司代码
- `document_number` - 凭证号
- `posting_date` - 过账日期
- `special_gl_indicator` - 特殊总账标识
- `line_item_number` - 行项目号
- `local_amount` - 金额

**问题类型**:
- `missing_business_partner` - 特殊总账项目缺少业务伙伴
- `bill_missing_maturity_date` - 票据缺少到期日
- `long_outstanding` - 长期未清项目（超过1年）

**使用示例**:

```sql
-- 查询所有数据质量问题
SELECT
    issue_type,
    issue_description,
    COUNT(*) as count,
    SUM(local_amount) as total_amount
FROM v_special_gl_data_quality
GROUP BY issue_type, issue_description
ORDER BY count DESC;

-- 查询缺少业务伙伴的项目
SELECT
    document_number,
    posting_date,
    special_gl_indicator,
    local_amount
FROM v_special_gl_data_quality
WHERE issue_type = 'missing_business_partner'
ORDER BY posting_date DESC;

-- 查询票据缺少到期日的项目
SELECT
    document_number,
    posting_date,
    local_amount
FROM v_special_gl_data_quality
WHERE issue_type = 'bill_missing_maturity_date'
ORDER BY local_amount DESC;
```

---

### 12. mv_special_gl_balance - 特殊总账余额物化视图

**用途**: 快速查询余额（需定期刷新）

**字段说明**:
- `company_code` - 公司代码
- `special_gl_indicator` - 特殊总账标识
- `account_id` - 科目代码
- `business_partner` - 业务伙伴
- `transaction_count` - 交易笔数
- `open_balance` - 未清余额
- `last_posting_date` - 最后过账日期
- `snapshot_date` - 快照日期

**刷新方法**:

```sql
-- 手动刷新物化视图
SELECT refresh_special_gl_materialized_views();

-- 或直接刷新
REFRESH MATERIALIZED VIEW CONCURRENTLY mv_special_gl_balance;
```

**使用示例**:

```sql
-- 快速查询余额
SELECT
    business_partner,
    special_gl_indicator,
    open_balance,
    last_posting_date
FROM mv_special_gl_balance
WHERE company_code = '1000'
ORDER BY open_balance DESC;

-- 按类型汇总余额
SELECT
    special_gl_indicator,
    SUM(open_balance) as total_balance,
    SUM(transaction_count) as total_transactions
FROM mv_special_gl_balance
GROUP BY special_gl_indicator;
```

---

## 🔧 维护工具

### 刷新物化视图

```sql
-- 刷新所有特殊总账物化视图
SELECT refresh_special_gl_materialized_views();
```

### 收集统计信息

```sql
-- 收集统计信息以优化查询性能
SELECT analyze_special_gl_tables();
```

---

## 📈 常用报表查询

### 1. 资产负债表 - 预付款/预收款

```sql
-- 预付款（资产）
SELECT
    '预付账款' as account_name,
    SUM(net_open_balance) as balance
FROM v_down_payment_balance
WHERE company_code = '1000';

-- 预收款（负债）
SELECT
    '预收账款' as account_name,
    SUM(net_open_balance) as balance
FROM v_advance_payment_balance
WHERE company_code = '1000';
```

### 2. 特殊总账月度报表

```sql
SELECT
    fiscal_period as month,
    special_gl_type,
    transaction_count,
    period_amount,
    open_at_period_end
FROM v_special_gl_monthly_trend
WHERE fiscal_year = 2026
  AND company_code = '1000'
ORDER BY fiscal_period, special_gl_type;
```

### 3. 风险管理报表

```sql
SELECT
    risk_level,
    special_gl_type,
    COUNT(*) as count,
    SUM(local_amount) as total_amount
FROM v_special_gl_risk_alert
WHERE company_code = '1000'
GROUP BY risk_level, special_gl_type
ORDER BY
    CASE risk_level
        WHEN 'HIGH' THEN 1
        WHEN 'MEDIUM' THEN 2
        WHEN 'LOW' THEN 3
    END,
    special_gl_type;
```

### 4. 清账效率KPI报表

```sql
SELECT
    fiscal_year,
    special_gl_type,
    clearing_rate_percent as clearing_rate,
    avg_clearing_days,
    CASE
        WHEN clearing_rate_percent >= 90 THEN '优秀'
        WHEN clearing_rate_percent >= 80 THEN '良好'
        WHEN clearing_rate_percent >= 70 THEN '一般'
        ELSE '需改进'
    END as performance_rating
FROM v_special_gl_clearing_efficiency
ORDER BY fiscal_year DESC, clearing_rate_percent DESC;
```

---

## ⚠️ 注意事项

1. **物化视图刷新**: `mv_special_gl_balance` 需要定期刷新（建议每日刷新）
2. **性能优化**: 大数据量查询建议使用物化视图
3. **索引维护**: 定期执行 `analyze_special_gl_tables()` 收集统计信息
4. **数据质量**: 定期检查 `v_special_gl_data_quality` 视图
5. **风险监控**: 每日检查 `v_special_gl_risk_alert` 视图

---

## 🔗 相关文档

- [UMSKZ 实施总结](./UMSKZ_IMPLEMENTATION_SUMMARY.md)
- [UMSKZ 快速参考](./UMSKZ_QUICK_REFERENCE.md)
- [数据库迁移脚本](./apps/fi/gl-service/migrations/20260118000001_add_special_gl_indicator.sql)
