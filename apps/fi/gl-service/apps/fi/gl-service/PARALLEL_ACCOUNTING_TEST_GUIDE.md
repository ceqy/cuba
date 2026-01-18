# 并行会计（Parallel Accounting）测试指南

## 概述

本文档提供并行会计功能的完整测试指南，包括数据库 migration、功能测试和验证步骤。

---

## 1. 数据库 Migration

### 1.1 准备数据库

```bash
# 创建测试数据库（如果不存在）
createdb gl_test_db

# 或使用 Docker
docker run -d \
  --name postgres-gl \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=gl_test_db \
  -p 5432:5432 \
  postgres:15
```

### 1.2 配置环境变量

```bash
# 创建 .env 文件
cat > .env <<EOF
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/gl_test_db
PORT=50060
RUST_LOG=info
COA_SERVICE_URL=http://localhost:50065
CHART_OF_ACCOUNTS=CN01
EOF
```

### 1.3 运行 Migration

```bash
# 方法 1: 使用 sqlx-cli
cd apps/fi/gl-service
sqlx migrate run

# 方法 2: 直接使用 psql
psql -d gl_test_db -f migrations/20240201000000_create_journal_entries.sql
psql -d gl_test_db -f migrations/20260118000000_add_parallel_accounting_ledger.sql

# 验证 migration 成功
psql -d gl_test_db -c "\d journal_entries"
psql -d gl_test_db -c "\d journal_entry_lines"
```

### 1.4 验证数据库 Schema

```sql
-- 检查新增字段
SELECT column_name, data_type, column_default
FROM information_schema.columns
WHERE table_name = 'journal_entries'
  AND column_name IN ('ledger_group', 'default_ledger');

-- 检查行项目表字段
SELECT column_name, data_type, column_default
FROM information_schema.columns
WHERE table_name = 'journal_entry_lines'
  AND column_name IN ('ledger', 'ledger_type', 'ledger_amount');

-- 检查索引
SELECT indexname, indexdef
FROM pg_indexes
WHERE tablename IN ('journal_entries', 'journal_entry_lines')
  AND indexname LIKE '%ledger%';

-- 检查视图
SELECT viewname FROM pg_views WHERE viewname = 'v_parallel_accounting_summary';

-- 检查余额表
SELECT * FROM ledger_balances LIMIT 1;
```

---

## 2. 启动服务

### 2.1 启动 GL Service

```bash
cd apps/fi/gl-service
cargo run

# 或使用 release 模式
cargo run --release

# 验证服务启动
curl http://localhost:50060/health || echo "Service started on port 50060"
```

### 2.2 启动其他 FI 服务（可选）

```bash
# 启动 AP Service
cd apps/fi/ap-service && cargo run &

# 启动 AR Service
cd apps/fi/ar-service && cargo run &

# 启动 COA Service
cd apps/fi/coa-service && cargo run &
```

---

## 3. 功能测试

### 3.1 测试 1: 默认主分类账（向后兼容性）

**目的**: 验证不传 ledger 参数时，自动使用主分类账 "0L"

```bash
# 使用 grpcurl 测试
grpcurl -plaintext -d '{
  "header": {
    "company_code": "1000",
    "document_type": "SA",
    "document_date": {"seconds": 1705536000},
    "posting_date": {"seconds": 1705536000},
    "fiscal_year": 2024,
    "fiscal_period": 1,
    "currency": "CNY",
    "reference_document": "TEST-001",
    "header_text": "测试凭证 - 默认主分类账"
  },
  "line_items": [
    {
      "line_item_number": 1,
      "posting_key": "40",
      "debit_credit_indicator": "S",
      "gl_account": "1001",
      "amount_in_local_currency": {"value": "1000.00", "currency_code": "CNY"}
    },
    {
      "line_item_number": 2,
      "posting_key": "50",
      "debit_credit_indicator": "H",
      "gl_account": "4001",
      "amount_in_local_currency": {"value": "1000.00", "currency_code": "CNY"}
    }
  ],
  "post_immediately": true
}' localhost:50060 fi.gl.v1.GlJournalEntryService/CreateJournalEntry
```

**预期结果**:
- ✅ 凭证创建成功
- ✅ `default_ledger` = "0L"
- ✅ 所有行项目的 `ledger` = "0L"
- ✅ 所有行项目的 `ledger_type` = 1 (Leading)

**验证数据库**:
```sql
-- 查询最新创建的凭证
SELECT id, document_number, default_ledger
FROM journal_entries
ORDER BY created_at DESC LIMIT 1;

-- 查询行项目的分类账信息
SELECT line_number, account_id, ledger, ledger_type, amount, ledger_amount
FROM journal_entry_lines
WHERE journal_entry_id = (
  SELECT id FROM journal_entries ORDER BY created_at DESC LIMIT 1
);
```

---

### 3.2 测试 2: 指定分类账（IFRS - 1L）

**目的**: 验证可以指定非主分类账进行记账

```bash
grpcurl -plaintext -d '{
  "header": {
    "company_code": "1000",
    "document_type": "SA",
    "document_date": {"seconds": 1705536000},
    "posting_date": {"seconds": 1705536000},
    "fiscal_year": 2024,
    "fiscal_period": 1,
    "currency": "CNY",
    "reference_document": "TEST-002",
    "header_text": "测试凭证 - IFRS 分类账",
    "default_ledger": "1L"
  },
  "line_items": [
    {
      "line_item_number": 1,
      "posting_key": "40",
      "debit_credit_indicator": "S",
      "gl_account": "1001",
      "amount_in_local_currency": {"value": "2000.00", "currency_code": "CNY"},
      "ledger": "1L",
      "ledger_type": 2,
      "amount_in_ledger_currency": {"value": "2100.00", "currency_code": "CNY"}
    },
    {
      "line_item_number": 2,
      "posting_key": "50",
      "debit_credit_indicator": "H",
      "gl_account": "4001",
      "amount_in_local_currency": {"value": "2000.00", "currency_code": "CNY"},
      "ledger": "1L",
      "ledger_type": 2,
      "amount_in_ledger_currency": {"value": "2100.00", "currency_code": "CNY"}
    }
  ],
  "post_immediately": true
}' localhost:50060 fi.gl.v1.GlJournalEntryService/CreateJournalEntry
```

**预期结果**:
- ✅ 凭证创建成功
- ✅ `default_ledger` = "1L"
- ✅ 所有行项目的 `ledger` = "1L"
- ✅ 所有行项目的 `ledger_type` = 2 (NonLeading)
- ✅ `ledger_amount` 可以与 `amount` 不同（IFRS 估值差异）

**验证数据库**:
```sql
SELECT
  je.document_number,
  je.default_ledger,
  jel.line_number,
  jel.account_id,
  jel.ledger,
  jel.ledger_type,
  jel.amount as gaap_amount,
  jel.ledger_amount as ifrs_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.default_ledger = '1L'
ORDER BY je.created_at DESC, jel.line_number
LIMIT 10;
```

---

### 3.3 测试 3: 并行会计（同时记录多个分类账）

**目的**: 验证同一笔业务可以在多个分类账中记录

```bash
# 创建主分类账凭证 (0L - 本地 GAAP)
grpcurl -plaintext -d '{
  "header": {
    "company_code": "1000",
    "document_type": "SA",
    "document_date": {"seconds": 1705536000},
    "posting_date": {"seconds": 1705536000},
    "fiscal_year": 2024,
    "fiscal_period": 1,
    "currency": "CNY",
    "reference_document": "TEST-003-0L",
    "header_text": "并行会计测试 - 本地 GAAP",
    "default_ledger": "0L"
  },
  "line_items": [
    {
      "line_item_number": 1,
      "debit_credit_indicator": "S",
      "gl_account": "1001",
      "amount_in_local_currency": {"value": "5000.00", "currency_code": "CNY"},
      "ledger": "0L",
      "ledger_type": 1
    },
    {
      "line_item_number": 2,
      "debit_credit_indicator": "H",
      "gl_account": "4001",
      "amount_in_local_currency": {"value": "5000.00", "currency_code": "CNY"},
      "ledger": "0L",
      "ledger_type": 1
    }
  ],
  "post_immediately": true
}' localhost:50060 fi.gl.v1.GlJournalEntryService/CreateJournalEntry

# 创建 IFRS 分类账凭证 (1L)
grpcurl -plaintext -d '{
  "header": {
    "company_code": "1000",
    "document_type": "SA",
    "document_date": {"seconds": 1705536000},
    "posting_date": {"seconds": 1705536000},
    "fiscal_year": 2024,
    "fiscal_period": 1,
    "currency": "CNY",
    "reference_document": "TEST-003-1L",
    "header_text": "并行会计测试 - IFRS",
    "default_ledger": "1L"
  },
  "line_items": [
    {
      "line_item_number": 1,
      "debit_credit_indicator": "S",
      "gl_account": "1001",
      "amount_in_local_currency": {"value": "5000.00", "currency_code": "CNY"},
      "ledger": "1L",
      "ledger_type": 2,
      "amount_in_ledger_currency": {"value": "5200.00", "currency_code": "CNY"}
    },
    {
      "line_item_number": 2,
      "debit_credit_indicator": "H",
      "gl_account": "4001",
      "amount_in_local_currency": {"value": "5000.00", "currency_code": "CNY"},
      "ledger": "1L",
      "ledger_type": 2,
      "amount_in_ledger_currency": {"value": "5200.00", "currency_code": "CNY"}
    }
  ],
  "post_immediately": true
}' localhost:50060 fi.gl.v1.GlJournalEntryService/CreateJournalEntry
```

**验证并行会计汇总视图**:
```sql
-- 查看并行会计汇总
SELECT
  company_code,
  fiscal_year,
  fiscal_period,
  ledger,
  CASE ledger_type
    WHEN 1 THEN '主分类账'
    WHEN 2 THEN '非主分类账'
    WHEN 3 THEN '扩展分类账'
  END as ledger_type_name,
  account_id,
  debit_credit,
  total_amount,
  total_local_amount,
  transaction_count
FROM v_parallel_accounting_summary
WHERE company_code = '1000'
  AND fiscal_year = 2024
ORDER BY ledger, account_id;
```

---

### 3.4 测试 4: 按分类账过滤查询

**目的**: 验证可以按分类账过滤查询凭证

```bash
# 查询所有分类账的凭证
grpcurl -plaintext -d '{
  "company_code": "1000",
  "fiscal_year": 2024
}' localhost:50060 fi.gl.v1.GlJournalEntryService/ListJournalEntries

# 只查询主分类账 (0L) 的凭证
grpcurl -plaintext -d '{
  "company_code": "1000",
  "fiscal_year": 2024,
  "ledgers": ["0L"]
}' localhost:50060 fi.gl.v1.GlJournalEntryService/ListJournalEntries

# 只查询 IFRS 分类账 (1L) 的凭证
grpcurl -plaintext -d '{
  "company_code": "1000",
  "fiscal_year": 2024,
  "ledgers": ["1L"]
}' localhost:50060 fi.gl.v1.GlJournalEntryService/ListJournalEntries

# 查询多个分类账的凭证
grpcurl -plaintext -d '{
  "company_code": "1000",
  "fiscal_year": 2024,
  "ledgers": ["0L", "1L"]
}' localhost:50060 fi.gl.v1.GlJournalEntryService/ListJournalEntries
```

**验证数据库查询性能**:
```sql
-- 测试分类账索引效果
EXPLAIN ANALYZE
SELECT * FROM journal_entries
WHERE company_code = '1000'
  AND fiscal_year = 2024
  AND default_ledger = '0L';

-- 测试行项目分类账索引
EXPLAIN ANALYZE
SELECT * FROM journal_entry_lines
WHERE ledger = '1L'
  AND account_id = '1001';
```

---

### 3.5 测试 5: AP/AR 服务集成测试

**目的**: 验证 AP/AR 服务通过 cuba-finance 调用 GL 时的并行会计支持

```bash
# 测试 AP Service 创建供应商发票（默认使用 0L）
grpcurl -plaintext -d '{
  "company_code": "1000",
  "account_id": "VENDOR001",
  "items": [
    {
      "gl_account": "5001",
      "debit_credit_indicator": "S",
      "amount": {"value": "3000.00", "currency_code": "CNY"},
      "cost_center": "CC001",
      "item_text": "采购原材料"
    },
    {
      "gl_account": "2001",
      "debit_credit_indicator": "H",
      "amount": {"value": "3000.00", "currency_code": "CNY"},
      "item_text": "应付账款"
    }
  ]
}' localhost:50061 fi.ap.v1.AccountsReceivablePayableService/PostDocument

# 验证 GL 中创建的凭证使用了默认分类账
```

---

## 4. 数据验证

### 4.1 验证数据完整性

```sql
-- 检查所有凭证的分类账字段
SELECT
  COUNT(*) as total_entries,
  COUNT(CASE WHEN default_ledger IS NULL THEN 1 END) as null_ledger,
  COUNT(CASE WHEN default_ledger = '0L' THEN 1 END) as leading_ledger,
  COUNT(CASE WHEN default_ledger = '1L' THEN 1 END) as ifrs_ledger
FROM journal_entries;

-- 检查所有行项目的分类账字段
SELECT
  ledger,
  ledger_type,
  COUNT(*) as line_count,
  SUM(amount) as total_amount
FROM journal_entry_lines
GROUP BY ledger, ledger_type
ORDER BY ledger;

-- 检查分类账余额表
SELECT * FROM ledger_balances
WHERE company_code = '1000'
  AND fiscal_year = 2024
ORDER BY ledger, account_id;
```

### 4.2 验证约束和索引

```sql
-- 测试 ledger 格式约束
INSERT INTO journal_entry_lines (
  id, journal_entry_id, line_number, account_id,
  debit_credit, amount, local_amount, ledger, ledger_type
) VALUES (
  gen_random_uuid(),
  (SELECT id FROM journal_entries LIMIT 1),
  999, '9999', 'D', 100, 100,
  'XX', -- 无效格式，应该失败
  1
);
-- 预期: ERROR - 违反 chk_ledger_format 约束

-- 测试 ledger_type 约束
INSERT INTO journal_entry_lines (
  id, journal_entry_id, line_number, account_id,
  debit_credit, amount, local_amount, ledger, ledger_type
) VALUES (
  gen_random_uuid(),
  (SELECT id FROM journal_entries LIMIT 1),
  999, '9999', 'D', 100, 100, '0L',
  99 -- 无效类型，应该失败
);
-- 预期: ERROR - 违反 chk_ledger_type 约束
```

---

## 5. 性能测试

### 5.1 索引效果测试

```sql
-- 测试分类账索引性能
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM journal_entries
WHERE company_code = '1000'
  AND fiscal_year = 2024
  AND default_ledger = '0L'
LIMIT 100;

-- 测试复合索引性能
EXPLAIN (ANALYZE, BUFFERS)
SELECT jel.*
FROM journal_entry_lines jel
JOIN journal_entries je ON jel.journal_entry_id = je.id
WHERE jel.ledger = '1L'
  AND jel.account_id = '1001'
LIMIT 100;
```

### 5.2 并行会计汇总视图性能

```sql
-- 测试汇总视图查询性能
EXPLAIN (ANALYZE, BUFFERS)
SELECT * FROM v_parallel_accounting_summary
WHERE company_code = '1000'
  AND fiscal_year = 2024
  AND ledger IN ('0L', '1L');
```

---

## 6. 回归测试

### 6.1 向后兼容性测试

**验证点**:
- ✅ 现有代码不传 ledger 参数时，自动使用 "0L"
- ✅ 现有数据迁移后，所有记录的 ledger 字段都是 "0L"
- ✅ 现有查询不受影响（不传 ledgers 过滤参数时返回所有分类账）

```sql
-- 验证现有数据迁移
SELECT COUNT(*) FROM journal_entries WHERE default_ledger != '0L';
-- 预期: 0

SELECT COUNT(*) FROM journal_entry_lines WHERE ledger != '0L';
-- 预期: 0（如果是全新数据库）
```

---

## 7. 清理和重置

### 7.1 清理测试数据

```sql
-- 删除测试凭证
DELETE FROM journal_entries
WHERE reference_document LIKE 'TEST-%';

-- 清空所有数据（谨慎使用）
TRUNCATE TABLE journal_entry_lines CASCADE;
TRUNCATE TABLE journal_entries CASCADE;
TRUNCATE TABLE ledger_balances CASCADE;
```

### 7.2 回滚 Migration（如需要）

```bash
# 使用 sqlx-cli 回滚
sqlx migrate revert

# 或手动删除表
psql -d gl_test_db -c "DROP TABLE IF EXISTS ledger_balances CASCADE;"
psql -d gl_test_db -c "DROP VIEW IF EXISTS v_parallel_accounting_summary CASCADE;"
psql -d gl_test_db -c "ALTER TABLE journal_entry_lines DROP COLUMN IF EXISTS ledger CASCADE;"
```

---

## 8. 预期结果总结

### 8.1 成功标准

| 测试项 | 预期结果 | 状态 |
|--------|---------|------|
| Migration 执行 | 所有字段和索引创建成功 | ⏸️ |
| 默认分类账测试 | 自动使用 "0L" | ⏸️ |
| 指定分类账测试 | 可以使用 "1L", "2L" 等 | ⏸️ |
| 并行会计测试 | 同一业务可在多个分类账记录 | ⏸️ |
| 查询过滤测试 | 可按分类账过滤查询 | ⏸️ |
| 向后兼容性 | 现有代码无需修改 | ✅ |
| 数据完整性 | 所有约束和索引生效 | ⏸️ |
| 性能测试 | 索引提升查询性能 | ⏸️ |

### 8.2 已知限制

1. **分类账余额表更新**: 当前需要手动或定时任务更新，未实现实时触发器
2. **跨分类账查询**: 暂不支持一次查询返回多个分类账的对比数据
3. **分类账转换**: 暂不支持将已过账凭证从一个分类账转移到另一个分类账

---

## 9. 故障排查

### 9.1 常见问题

**问题 1: Migration 失败 - 字段已存在**
```
ERROR: column "ledger" already exists
```
**解决**: 字段已经存在，跳过此 migration 或先回滚

**问题 2: 约束违反 - 无效的 ledger 格式**
```
ERROR: new row violates check constraint "chk_ledger_format"
```
**解决**: 确保 ledger 字段格式为 "0L", "1L", "2L" 等

**问题 3: 服务启动失败 - 数据库连接**
```
ERROR: could not connect to database
```
**解决**: 检查 DATABASE_URL 环境变量和数据库服务状态

### 9.2 调试命令

```bash
# 检查服务日志
RUST_LOG=debug cargo run

# 检查数据库连接
psql -d gl_test_db -c "SELECT version();"

# 检查 migration 状态
sqlx migrate info

# 测试 gRPC 连接
grpcurl -plaintext localhost:50060 list
```

---

## 10. 下一步

- [ ] 补充集成测试用例
- [ ] 添加性能基准测试
- [ ] 实现分类账余额实时更新
- [ ] 添加跨分类账对比报表
- [ ] 完善 API 文档和示例

---

**文档版本**: 1.0
**最后更新**: 2026-01-18
**维护者**: FI Team
