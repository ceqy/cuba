#!/bin/bash

# CUBA ERP SQL 测试数据插入脚本
# 直接在数据库中插入测试数据,绕过 API 接口问题

set -e

echo "🚀 开始插入测试数据到数据库..."

# 颜色输出
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 1. 插入会计分录测试数据
echo -e "${BLUE}📝 步骤 1: 插入会计分录测试数据${NC}"
docker exec cuba-postgres psql -U postgres -d cuba_fi_gl << 'EOF'

-- 插入测试会计分录
INSERT INTO journal_entries (
  id, company_code, fiscal_year, fiscal_period, document_number,
  document_date, posting_date, entry_date, document_type,
  currency, reference, header_text, status, tenant_id,
  created_by, created_at
) VALUES
(
  gen_random_uuid(),
  '1000',
  2026,
  1,
  'JE-2026-001',
  '2026-01-20',
  '2026-01-20',
  '2026-01-20',
  'SA',
  'CNY',
  'TEST-001',
  '测试销售收入',
  'DRAFT',
  'default',
  '4c7c020c-e412-45f8-81d9-b043969fe0be',
  NOW()
),
(
  gen_random_uuid(),
  '1000',
  2026,
  1,
  'JE-2026-002',
  '2026-01-20',
  '2026-01-20',
  '2026-01-20',
  'KR',
  'CNY',
  'TEST-002',
  '测试采购成本',
  'POSTED',
  'default',
  '4c7c020c-e412-45f8-81d9-b043969fe0be',
  NOW()
),
(
  gen_random_uuid(),
  '1000',
  2026,
  1,
  'JE-2026-003',
  '2026-01-19',
  '2026-01-19',
  '2026-01-19',
  'SA',
  'CNY',
  'TEST-003',
  '测试服务收入',
  'POSTED',
  'default',
  '4c7c020c-e412-45f8-81d9-b043969fe0be',
  NOW() - INTERVAL '1 day'
);

-- 查看插入的数据
SELECT
  document_number,
  document_type,
  header_text,
  status,
  created_at::date as created_date
FROM journal_entries
ORDER BY created_at DESC;

EOF

echo -e "${GREEN}✅ 会计分录数据插入成功${NC}"

# 2. 插入会计分录明细行
echo -e "\n${BLUE}📝 步骤 2: 插入会计分录明细行${NC}"
docker exec cuba-postgres psql -U postgres -d cuba_fi_gl << 'EOF'

-- 获取刚插入的分录 ID
DO $$
DECLARE
  je1_id text;
  je2_id text;
  je3_id text;
BEGIN
  -- 获取分录 ID
  SELECT id INTO je1_id FROM journal_entries WHERE document_number = 'JE-2026-001';
  SELECT id INTO je2_id FROM journal_entries WHERE document_number = 'JE-2026-002';
  SELECT id INTO je3_id FROM journal_entries WHERE document_number = 'JE-2026-003';

  -- 插入 JE-2026-001 的明细行 (销售收入)
  INSERT INTO journal_entry_lines (
    id, journal_entry_id, line_number, account_code,
    debit_credit, amount, currency, text,
    cost_center, profit_center, created_at, updated_at
  ) VALUES
  (
    gen_random_uuid()::text,
    je1_id,
    1,
    '110000',
    'D',
    11300.00,
    'CNY',
    '应收账款',
    NULL,
    NULL,
    NOW(),
    NOW()
  ),
  (
    gen_random_uuid()::text,
    je1_id,
    2,
    '600000',
    'C',
    10000.00,
    'CNY',
    '主营业务收入',
    NULL,
    NULL,
    NOW(),
    NOW()
  ),
  (
    gen_random_uuid()::text,
    je1_id,
    3,
    '220300',
    'C',
    1300.00,
    'CNY',
    '销项税',
    NULL,
    NULL,
    NOW(),
    NOW()
  );

  -- 插入 JE-2026-002 的明细行 (采购成本)
  INSERT INTO journal_entry_lines (
    id, journal_entry_id, line_number, account_code,
    debit_credit, amount, currency, text,
    cost_center, profit_center, created_at, updated_at
  ) VALUES
  (
    gen_random_uuid()::text,
    je2_id,
    1,
    '500000',
    'D',
    5000.00,
    'CNY',
    '原材料采购',
    'CC001',
    NULL,
    NOW(),
    NOW()
  ),
  (
    gen_random_uuid()::text,
    je2_id,
    2,
    '210000',
    'C',
    5000.00,
    'CNY',
    '应付账款',
    NULL,
    NULL,
    NOW(),
    NOW()
  );

  -- 插入 JE-2026-003 的明细行 (服务收入)
  INSERT INTO journal_entry_lines (
    id, journal_entry_id, line_number, account_code,
    debit_credit, amount, currency, text,
    cost_center, profit_center, created_at, updated_at
  ) VALUES
  (
    gen_random_uuid()::text,
    je3_id,
    1,
    '110000',
    'D',
    8000.00,
    'CNY',
    '应收账款',
    NULL,
    NULL,
    NOW() - INTERVAL '1 day',
    NOW() - INTERVAL '1 day'
  ),
  (
    gen_random_uuid()::text,
    je3_id,
    2,
    '610000',
    'C',
    8000.00,
    'CNY',
    '服务收入',
    NULL,
    'PC001',
    NOW() - INTERVAL '1 day',
    NOW() - INTERVAL '1 day'
  );

END $$;

-- 查看插入的明细行
SELECT
  je.document_number,
  jel.line_number,
  jel.account_code,
  jel.debit_credit,
  jel.amount,
  jel.text
FROM journal_entry_lines jel
JOIN journal_entries je ON jel.journal_entry_id = je.id
ORDER BY je.document_number, jel.line_number;

EOF

echo -e "${GREEN}✅ 会计分录明细行插入成功${NC}"

# 3. 验证数据
echo -e "\n${BLUE}📝 步骤 3: 验证插入的数据${NC}"
docker exec cuba-postgres psql -U postgres -d cuba_fi_gl << 'EOF'

-- 统计数据
SELECT
  '会计分录总数' as item,
  COUNT(*)::text as count
FROM journal_entries
UNION ALL
SELECT
  '明细行总数' as item,
  COUNT(*)::text as count
FROM journal_entry_lines
UNION ALL
SELECT
  '草稿状态' as item,
  COUNT(*)::text as count
FROM journal_entries
WHERE status = 'DRAFT'
UNION ALL
SELECT
  '已过账' as item,
  COUNT(*)::text as count
FROM journal_entries
WHERE status = 'POSTED';

EOF

echo -e "${GREEN}✅ 数据验证完成${NC}"

# 4. 输出总结
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ 测试数据插入完成!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

echo -e "\n${BLUE}📊 已插入的测试数据:${NC}"
echo -e "  ✓ 会计分录: 3 条"
echo -e "    - JE-2026-001: 销售收入 (草稿)"
echo -e "    - JE-2026-002: 采购成本 (已过账)"
echo -e "    - JE-2026-003: 服务收入 (已过账)"
echo -e "  ✓ 明细行: 8 条"

echo -e "\n${BLUE}🌐 现在可以在 Swagger UI 中测试查询接口:${NC}"
echo -e "  ${GREEN}http://localhost:8081${NC}"

echo -e "\n${BLUE}💡 测试建议:${NC}"
echo -e "  1. 使用 demo_user 账号登录"
echo -e "  2. 测试 GET /api/v1/finance/gl/journal-entries/list"
echo -e "  3. 测试 GET /api/v1/finance/gl/journal-entries/{id}"
echo -e "  4. 查看不同状态的分录 (DRAFT vs POSTED)"

echo -e "\n${YELLOW}⚠️  注意:${NC}"
echo -e "  - 这些数据是直接插入数据库的"
echo -e "  - 如果 API 创建接口修复后,建议使用 API 创建数据"
echo -e "  - 当前数据仅用于测试查询功能"

echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
