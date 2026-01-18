# UMSKZ 数据库迁移执行指南

## 📋 概述

本文档提供 UMSKZ (特殊总账标识) 数据库迁移的详细执行步骤和验证方法。

---

## 🎯 迁移内容

### 新增内容
- ✅ 1 个字段：`special_gl_indicator`
- ✅ 2 个索引：单列索引 + 复合索引
- ✅ 1 个约束：数据完整性检查
- ✅ 13 个视图：业务分析视图
- ✅ 1 个物化视图：性能优化
- ✅ 2 个函数：维护工具

### 支持的特殊总账类型
- `A` = 票据 (Bills of Exchange)
- `F` = 预付款 (Down Payment)
- `V` = 预收款 (Advance Payment)
- `W` = 票据贴现 (Bill of Exchange Discount)
- 空值 = 普通业务

---

## 🚀 执行步骤

### 步骤 1: 备份数据库

**重要**: 在执行迁移前，务必备份数据库！

```bash
# 备份整个数据库
pg_dump -h localhost -U postgres -d gl_service > backup_before_umskz_$(date +%Y%m%d_%H%M%S).sql

# 或只备份相关表
pg_dump -h localhost -U postgres -d gl_service \
  -t journal_entries \
  -t journal_entry_lines \
  > backup_tables_before_umskz_$(date +%Y%m%d_%H%M%S).sql
```

### 步骤 2: 检查数据库连接

```bash
# 测试数据库连接
psql -h localhost -U postgres -d gl_service -c "SELECT version();"
```

### 步骤 3: 执行迁移脚本

```bash
# 执行迁移
psql -h localhost -U postgres -d gl_service \
  -f apps/fi/gl-service/migrations/20260118000001_add_special_gl_indicator.sql

# 或使用 sqlx 迁移工具
sqlx migrate run --database-url "postgresql://postgres:password@localhost/gl_service"
```

### 步骤 4: 验证迁移结果

执行以下 SQL 验证迁移是否成功：

```sql
-- 1. 检查字段是否添加成功
SELECT
    column_name,
    data_type,
    character_maximum_length,
    column_default,
    is_nullable
FROM information_schema.columns
WHERE table_name = 'journal_entry_lines'
  AND column_name = 'special_gl_indicator';

-- 预期结果：
-- column_name: special_gl_indicator
-- data_type: character varying
-- character_maximum_length: 1
-- column_default: ''::character varying
-- is_nullable: YES

-- 2. 检查约束是否创建成功
SELECT
    constraint_name,
    constraint_type
FROM information_schema.table_constraints
WHERE table_name = 'journal_entry_lines'
  AND constraint_name = 'chk_special_gl_indicator';

-- 预期结果：
-- constraint_name: chk_special_gl_indicator
-- constraint_type: CHECK

-- 3. 检查索引是否创建成功
SELECT
    indexname,
    indexdef
FROM pg_indexes
WHERE tablename = 'journal_entry_lines'
  AND indexname LIKE '%special_gl%';

-- 预期结果：应该看到 2 个索引
-- idx_journal_entry_lines_special_gl
-- idx_journal_lines_account_special_gl

-- 4. 检查视图是否创建成功
SELECT
    table_name,
    table_type
FROM information_schema.tables
WHERE table_schema = 'public'
  AND table_name LIKE 'v_special_gl%'
ORDER BY table_name;

-- 预期结果：应该看到 13 个视图

-- 5. 检查物化视图是否创建成功
SELECT
    matviewname,
    definition
FROM pg_matviews
WHERE matviewname = 'mv_special_gl_balance';

-- 预期结果：应该看到 1 个物化视图

-- 6. 检查函数是否创建成功
SELECT
    routine_name,
    routine_type
FROM information_schema.routines
WHERE routine_schema = 'public'
  AND routine_name IN (
      'refresh_special_gl_materialized_views',
      'analyze_special_gl_tables'
  );

-- 预期结果：应该看到 2 个函数
```

---

## ✅ 功能验证

### 验证 1: 插入测试数据

```sql
-- 开始事务
BEGIN;

-- 创建测试凭证
INSERT INTO journal_entries (
    id,
    company_code,
    fiscal_year,
    fiscal_period,
    posting_date,
    document_date,
    status,
    currency,
    created_at
) VALUES (
    gen_random_uuid(),
    '1000',
    2026,
    1,
    '2026-01-18',
    '2026-01-18',
    'POSTED',
    'CNY',
    NOW()
) RETURNING id;

-- 记录返回的 ID，用于下面的插入
-- 假设返回的 ID 是: 'xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx'

-- 插入预付款行项目（F）
INSERT INTO journal_entry_lines (
    id,
    journal_entry_id,
    line_item_number,
    account_id,
    debit_credit,
    amount,
    local_amount,
    currency,
    special_gl_indicator
) VALUES (
    gen_random_uuid(),
    'xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx', -- 替换为上面的 ID
    1,
    '1100',
    'D',
    10000.00,
    10000.00,
    'CNY',
    'F' -- 预付款
);

-- 插入对应的贷方行项目
INSERT INTO journal_entry_lines (
    id,
    journal_entry_id,
    line_item_number,
    account_id,
    debit_credit,
    amount,
    local_amount,
    currency,
    special_gl_indicator
) VALUES (
    gen_random_uuid(),
    'xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx', -- 替换为上面的 ID
    2,
    '2100',
    'C',
    10000.00,
    10000.00,
    'CNY',
    '' -- 普通业务
);

-- 提交事务
COMMIT;
```

### 验证 2: 查询测试数据

```sql
-- 查询特殊总账项目
SELECT * FROM v_special_gl_items
WHERE special_gl_indicator = 'F'
ORDER BY posting_date DESC
LIMIT 5;

-- 查询预付款余额
SELECT * FROM v_down_payment_balance
WHERE company_code = '1000';

-- 查询特殊总账汇总
SELECT * FROM v_special_gl_summary
WHERE fiscal_year = 2026
  AND fiscal_period = 1;
```

### 验证 3: 测试约束

```sql
-- 测试有效值（应该成功）
BEGIN;
UPDATE journal_entry_lines
SET special_gl_indicator = 'A'
WHERE id = (SELECT id FROM journal_entry_lines LIMIT 1);
ROLLBACK;

-- 测试无效值（应该失败）
BEGIN;
UPDATE journal_entry_lines
SET special_gl_indicator = 'X' -- 无效值
WHERE id = (SELECT id FROM journal_entry_lines LIMIT 1);
-- 预期错误: new row for relation "journal_entry_lines" violates check constraint "chk_special_gl_indicator"
ROLLBACK;
```

### 验证 4: 测试物化视图刷新

```sql
-- 刷新物化视图
SELECT refresh_special_gl_materialized_views();

-- 查询物化视图
SELECT * FROM mv_special_gl_balance
WHERE company_code = '1000'
LIMIT 10;
```

### 验证 5: 测试维护函数

```sql
-- 收集统计信息
SELECT analyze_special_gl_tables();

-- 检查统计信息
SELECT
    schemaname,
    tablename,
    last_analyze,
    last_autoanalyze
FROM pg_stat_user_tables
WHERE tablename = 'journal_entry_lines';
```

---

## 📊 性能验证

### 验证索引效果

```sql
-- 查看查询计划（应该使用索引）
EXPLAIN ANALYZE
SELECT * FROM journal_entry_lines
WHERE special_gl_indicator = 'F';

-- 预期结果：应该看到 "Index Scan" 或 "Bitmap Index Scan"
-- 而不是 "Seq Scan"

-- 查看复合索引效果
EXPLAIN ANALYZE
SELECT * FROM journal_entry_lines
WHERE account_id = '1100'
  AND special_gl_indicator = 'F';

-- 预期结果：应该使用 idx_journal_lines_account_special_gl 索引
```

### 验证视图性能

```sql
-- 测试视图查询性能
EXPLAIN ANALYZE
SELECT * FROM v_special_gl_items
WHERE fiscal_year = 2026
  AND special_gl_indicator = 'F';

-- 测试汇总视图性能
EXPLAIN ANALYZE
SELECT * FROM v_special_gl_summary
WHERE fiscal_year = 2026;

-- 测试物化视图性能（应该最快）
EXPLAIN ANALYZE
SELECT * FROM mv_special_gl_balance
WHERE company_code = '1000';
```

---

## 🔄 回滚步骤

如果迁移出现问题，可以使用以下步骤回滚：

### 方法 1: 使用备份恢复

```bash
# 恢复整个数据库
psql -h localhost -U postgres -d gl_service < backup_before_umskz_YYYYMMDD_HHMMSS.sql
```

### 方法 2: 手动回滚

```sql
-- 1. 删除物化视图
DROP MATERIALIZED VIEW IF EXISTS mv_special_gl_balance;

-- 2. 删除函数
DROP FUNCTION IF EXISTS refresh_special_gl_materialized_views();
DROP FUNCTION IF EXISTS analyze_special_gl_tables();

-- 3. 删除视图
DROP VIEW IF EXISTS v_special_gl_data_quality;
DROP VIEW IF EXISTS v_special_gl_risk_alert;
DROP VIEW IF EXISTS v_business_partner_special_gl;
DROP VIEW IF EXISTS v_special_gl_clearing_efficiency;
DROP VIEW IF EXISTS v_special_gl_monthly_trend;
DROP VIEW IF EXISTS v_special_gl_aging;
DROP VIEW IF EXISTS v_bill_maturity_analysis;
DROP VIEW IF EXISTS v_advance_payment_balance;
DROP VIEW IF EXISTS v_down_payment_balance;
DROP VIEW IF EXISTS v_special_gl_summary;
DROP VIEW IF EXISTS v_special_gl_items;

-- 4. 删除索引
DROP INDEX IF EXISTS idx_mv_special_gl_balance_partner;
DROP INDEX IF EXISTS idx_mv_special_gl_balance_company;
DROP INDEX IF EXISTS idx_journal_lines_account_special_gl;
DROP INDEX IF EXISTS idx_journal_entry_lines_special_gl;

-- 5. 删除约束
ALTER TABLE journal_entry_lines
DROP CONSTRAINT IF EXISTS chk_special_gl_indicator;

-- 6. 删除字段
ALTER TABLE journal_entry_lines
DROP COLUMN IF EXISTS special_gl_indicator;
```

---

## 📈 监控建议

### 1. 定期刷新物化视图

建议设置定时任务，每日刷新物化视图：

```sql
-- 创建定时刷新任务（使用 pg_cron 扩展）
SELECT cron.schedule(
    'refresh-special-gl-mv',
    '0 1 * * *', -- 每天凌晨1点执行
    $$SELECT refresh_special_gl_materialized_views();$$
);
```

或使用系统 cron：

```bash
# 添加到 crontab
0 1 * * * psql -h localhost -U postgres -d gl_service -c "SELECT refresh_special_gl_materialized_views();"
```

### 2. 定期收集统计信息

```bash
# 每周收集一次统计信息
0 2 * * 0 psql -h localhost -U postgres -d gl_service -c "SELECT analyze_special_gl_tables();"
```

### 3. 监控数据质量

```sql
-- 每日检查数据质量问题
SELECT
    issue_type,
    COUNT(*) as count
FROM v_special_gl_data_quality
GROUP BY issue_type;
```

### 4. 监控风险项目

```sql
-- 每日检查高风险项目
SELECT
    COUNT(*) as high_risk_count,
    SUM(local_amount) as high_risk_amount
FROM v_special_gl_risk_alert
WHERE risk_level = 'HIGH';
```

---

## ⚠️ 注意事项

1. **生产环境执行**:
   - 建议在业务低峰期执行
   - 提前通知相关人员
   - 准备回滚方案

2. **数据备份**:
   - 执行前必须备份
   - 验证备份可用性
   - 保留备份至少7天

3. **性能影响**:
   - 迁移过程可能锁表
   - 索引创建需要时间
   - 物化视图首次创建较慢

4. **应用兼容性**:
   - 确保应用代码已更新
   - 测试 gRPC 接口
   - 验证数据序列化

5. **监控告警**:
   - 设置数据质量告警
   - 设置风险项目告警
   - 监控查询性能

---

## 📞 故障排查

### 问题 1: 迁移脚本执行失败

**症状**: 执行迁移脚本时报错

**排查步骤**:
1. 检查数据库连接
2. 检查用户权限
3. 查看错误日志
4. 检查表是否存在

**解决方案**:
```sql
-- 检查表是否存在
SELECT tablename FROM pg_tables WHERE tablename = 'journal_entry_lines';

-- 检查用户权限
SELECT has_table_privilege('postgres', 'journal_entry_lines', 'ALTER');
```

### 问题 2: 约束冲突

**症状**: 插入数据时违反约束

**排查步骤**:
```sql
-- 检查约束定义
SELECT
    conname,
    pg_get_constraintdef(oid)
FROM pg_constraint
WHERE conname = 'chk_special_gl_indicator';

-- 检查违反约束的数据
SELECT *
FROM journal_entry_lines
WHERE special_gl_indicator NOT IN ('', 'A', 'F', 'V', 'W')
  AND special_gl_indicator IS NOT NULL;
```

### 问题 3: 视图查询慢

**症状**: 视图查询性能差

**排查步骤**:
```sql
-- 检查是否使用索引
EXPLAIN ANALYZE
SELECT * FROM v_special_gl_items
WHERE fiscal_year = 2026;

-- 检查统计信息是否过期
SELECT
    schemaname,
    tablename,
    last_analyze,
    last_autoanalyze
FROM pg_stat_user_tables
WHERE tablename IN ('journal_entries', 'journal_entry_lines');
```

**解决方案**:
```sql
-- 收集统计信息
SELECT analyze_special_gl_tables();

-- 或使用物化视图
SELECT * FROM mv_special_gl_balance;
```

### 问题 4: 物化视图刷新失败

**症状**: 刷新物化视图时报错

**排查步骤**:
```sql
-- 检查物化视图定义
SELECT definition FROM pg_matviews WHERE matviewname = 'mv_special_gl_balance';

-- 手动刷新
REFRESH MATERIALIZED VIEW mv_special_gl_balance;
```

---

## 📚 相关文档

- [UMSKZ 实施总结](./UMSKZ_IMPLEMENTATION_SUMMARY.md)
- [UMSKZ 快速参考](./UMSKZ_QUICK_REFERENCE.md)
- [数据库视图使用指南](./UMSKZ_DATABASE_VIEWS_GUIDE.md)
- [迁移脚本](./apps/fi/gl-service/migrations/20260118000001_add_special_gl_indicator.sql)

---

## ✅ 迁移检查清单

执行迁移前，请确认以下事项：

- [ ] 已备份数据库
- [ ] 已测试备份恢复
- [ ] 已通知相关人员
- [ ] 已准备回滚方案
- [ ] 已更新应用代码
- [ ] 已在测试环境验证
- [ ] 已选择业务低峰期
- [ ] 已准备监控工具

执行迁移后，请验证以下内容：

- [ ] 字段添加成功
- [ ] 约束创建成功
- [ ] 索引创建成功
- [ ] 视图创建成功
- [ ] 物化视图创建成功
- [ ] 函数创建成功
- [ ] 测试数据插入成功
- [ ] 视图查询正常
- [ ] 约束验证正常
- [ ] 性能符合预期
- [ ] 应用接口正常
- [ ] 监控告警正常

---

**迁移完成后，请保留本文档作为运维参考！**
