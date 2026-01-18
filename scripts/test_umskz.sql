-- ============================================================================
-- UMSKZ 特殊总账标识 - 测试查询脚本
-- 用途: 验证功能和提供查询示例
-- ============================================================================

\echo '=========================================='
\echo '  UMSKZ 特殊总账标识 - 测试查询'
\echo '=========================================='
\echo ''

-- ============================================================================
-- 1. 基础验证
-- ============================================================================

\echo '1. 检查字段是否存在...'
SELECT
    column_name,
    data_type,
    character_maximum_length,
    column_default
FROM information_schema.columns
WHERE table_name = 'journal_entry_lines'
  AND column_name = 'special_gl_indicator';

\echo ''
\echo '2. 检查约束是否创建...'
SELECT
    constraint_name,
    constraint_type
FROM information_schema.table_constraints
WHERE table_name = 'journal_entry_lines'
  AND constraint_name = 'chk_special_gl_indicator';

\echo ''
\echo '3. 检查索引是否创建...'
SELECT
    indexname,
    indexdef
FROM pg_indexes
WHERE tablename = 'journal_entry_lines'
  AND indexname LIKE '%special_gl%';

\echo ''
\echo '4. 检查视图是否创建...'
SELECT
    table_name,
    table_type
FROM information_schema.tables
WHERE table_schema = 'public'
  AND table_name LIKE 'v_special_gl%'
ORDER BY table_name;

-- ============================================================================
-- 2. 插入测试数据
-- ============================================================================

\echo ''
\echo '=========================================='
\echo '  插入测试数据'
\echo '=========================================='
\echo ''

BEGIN;

-- 创建测试凭证 1: 预付款
\echo '创建测试凭证 1: 预付款...'
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
    '11111111-1111-1111-1111-111111111111',
    '1000',
    2026,
    1,
    '2026-01-18',
    '2026-01-18',
    'POSTED',
    'CNY',
    NOW()
);

INSERT INTO journal_entry_lines (
    id,
    journal_entry_id,
    line_item_number,
    account_id,
    debit_credit,
    amount,
    local_amount,
    currency,
    special_gl_indicator,
    business_partner
) VALUES
(
    gen_random_uuid(),
    '11111111-1111-1111-1111-111111111111',
    1,
    '1100',
    'D',
    10000.00,
    10000.00,
    'CNY',
    'F', -- 预付款
    'VENDOR001'
),
(
    gen_random_uuid(),
    '11111111-1111-1111-1111-111111111111',
    2,
    '2100',
    'C',
    10000.00,
    10000.00,
    'CNY',
    '', -- 普通业务
    NULL
);

-- 创建测试凭证 2: 票据
\echo '创建测试凭证 2: 票据...'
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
    '22222222-2222-2222-2222-222222222222',
    '1000',
    2026,
    1,
    '2026-01-18',
    '2026-01-18',
    'POSTED',
    'CNY',
    NOW()
);

INSERT INTO journal_entry_lines (
    id,
    journal_entry_id,
    line_item_number,
    account_id,
    debit_credit,
    amount,
    local_amount,
    currency,
    special_gl_indicator,
    business_partner,
    clearing_date
) VALUES
(
    gen_random_uuid(),
    '22222222-2222-2222-2222-222222222222',
    1,
    '1120',
    'D',
    50000.00,
    50000.00,
    'CNY',
    'A', -- 票据
    'CUSTOMER001',
    '2026-04-18' -- 90天后到期
),
(
    gen_random_uuid(),
    '22222222-2222-2222-2222-222222222222',
    2,
    '4000',
    'C',
    50000.00,
    50000.00,
    'CNY',
    '', -- 普通业务
    NULL,
    NULL
);

-- 创建测试凭证 3: 预收款
\echo '创建测试凭证 3: 预收款...'
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
    '33333333-3333-3333-3333-333333333333',
    '1000',
    2026,
    1,
    '2026-01-18',
    '2026-01-18',
    'POSTED',
    'CNY',
    NOW()
);

INSERT INTO journal_entry_lines (
    id,
    journal_entry_id,
    line_item_number,
    account_id,
    debit_credit,
    amount,
    local_amount,
    currency,
    special_gl_indicator,
    business_partner
) VALUES
(
    gen_random_uuid(),
    '33333333-3333-3333-3333-333333333333',
    1,
    '2100',
    'D',
    20000.00,
    20000.00,
    'CNY',
    '', -- 普通业务
    NULL
),
(
    gen_random_uuid(),
    '33333333-3333-3333-3333-333333333333',
    2,
    '2200',
    'C',
    20000.00,
    20000.00,
    'CNY',
    'V', -- 预收款
    'CUSTOMER002'
);

COMMIT;

\echo ''
\echo '测试数据插入成功！'

-- ============================================================================
-- 3. 查询测试
-- ============================================================================

\echo ''
\echo '=========================================='
\echo '  查询测试'
\echo '=========================================='
\echo ''

\echo '1. 查询所有特殊总账项目...'
SELECT
    document_number,
    special_gl_indicator,
    special_gl_description,
    business_partner,
    local_amount,
    clearing_status
FROM v_special_gl_items
ORDER BY posting_date DESC;

\echo ''
\echo '2. 查询预付款余额...'
SELECT
    vendor_code,
    net_open_balance,
    transaction_count
FROM v_down_payment_balance
WHERE company_code = '1000';

\echo ''
\echo '3. 查询预收款余额...'
SELECT
    customer_code,
    net_open_balance,
    transaction_count
FROM v_advance_payment_balance
WHERE company_code = '1000';

\echo ''
\echo '4. 查询票据到期分析...'
SELECT
    document_number,
    business_partner,
    local_amount,
    maturity_status,
    days_to_maturity
FROM v_bill_maturity_analysis
ORDER BY clearing_date NULLS LAST;

\echo ''
\echo '5. 查询特殊总账汇总...'
SELECT
    special_gl_indicator,
    special_gl_description,
    transaction_count,
    total_local_amount,
    open_amount,
    cleared_amount
FROM v_special_gl_summary
WHERE fiscal_year = 2026
  AND fiscal_period = 1;

\echo ''
\echo '6. 查询业务伙伴特殊总账汇总...'
SELECT
    business_partner,
    special_gl_type,
    transaction_count,
    total_amount,
    open_amount
FROM v_business_partner_special_gl
WHERE company_code = '1000'
ORDER BY business_partner, special_gl_type;

-- ============================================================================
-- 4. 约束测试
-- ============================================================================

\echo ''
\echo '=========================================='
\echo '  约束测试'
\echo '=========================================='
\echo ''

\echo '测试有效值（应该成功）...'
BEGIN;
UPDATE journal_entry_lines
SET special_gl_indicator = 'A'
WHERE journal_entry_id = '11111111-1111-1111-1111-111111111111'
  AND line_item_number = 1;
\echo '✓ 有效值测试通过'
ROLLBACK;

\echo ''
\echo '测试无效值（应该失败）...'
BEGIN;
DO $$
BEGIN
    UPDATE journal_entry_lines
    SET special_gl_indicator = 'X' -- 无效值
    WHERE journal_entry_id = '11111111-1111-1111-1111-111111111111'
      AND line_item_number = 1;
    RAISE EXCEPTION '约束测试失败：应该拒绝无效值';
EXCEPTION
    WHEN check_violation THEN
        RAISE NOTICE '✓ 约束测试通过：成功拒绝无效值';
END $$;
ROLLBACK;

-- ============================================================================
-- 5. 性能测试
-- ============================================================================

\echo ''
\echo '=========================================='
\echo '  性能测试'
\echo '=========================================='
\echo ''

\echo '测试索引使用情况...'
EXPLAIN ANALYZE
SELECT * FROM journal_entry_lines
WHERE special_gl_indicator = 'F';

\echo ''
\echo '测试复合索引使用情况...'
EXPLAIN ANALYZE
SELECT * FROM journal_entry_lines
WHERE account_id = '1100'
  AND special_gl_indicator = 'F';

\echo ''
\echo '测试视图查询性能...'
EXPLAIN ANALYZE
SELECT * FROM v_special_gl_items
WHERE fiscal_year = 2026
  AND special_gl_indicator = 'F';

-- ============================================================================
-- 6. 物化视图测试
-- ============================================================================

\echo ''
\echo '=========================================='
\echo '  物化视图测试'
\echo '=========================================='
\echo ''

\echo '刷新物化视图...'
SELECT refresh_special_gl_materialized_views();

\echo ''
\echo '查询物化视图...'
SELECT
    special_gl_indicator,
    business_partner,
    open_balance,
    transaction_count
FROM mv_special_gl_balance
WHERE company_code = '1000'
ORDER BY open_balance DESC;

-- ============================================================================
-- 7. 维护函数测试
-- ============================================================================

\echo ''
\echo '=========================================='
\echo '  维护函数测试'
\echo '=========================================='
\echo ''

\echo '收集统计信息...'
SELECT analyze_special_gl_tables();

\echo ''
\echo '检查统计信息...'
SELECT
    schemaname,
    tablename,
    last_analyze,
    last_autoanalyze
FROM pg_stat_user_tables
WHERE tablename = 'journal_entry_lines';

-- ============================================================================
-- 8. 数据质量检查
-- ============================================================================

\echo ''
\echo '=========================================='
\echo '  数据质量检查'
\echo '=========================================='
\echo ''

\echo '检查数据质量问题...'
SELECT
    issue_type,
    issue_description,
    COUNT(*) as count
FROM v_special_gl_data_quality
GROUP BY issue_type, issue_description
ORDER BY count DESC;

-- ============================================================================
-- 9. 风险预警检查
-- ============================================================================

\echo ''
\echo '=========================================='
\echo '  风险预警检查'
\echo '=========================================='
\echo ''

\echo '检查风险项目...'
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

-- ============================================================================
-- 10. 清理测试数据
-- ============================================================================

\echo ''
\echo '=========================================='
\echo '  清理测试数据'
\echo '=========================================='
\echo ''

\echo '是否清理测试数据？(y/N)'
\prompt '请输入: ' cleanup_choice

\if :cleanup_choice = 'y'
    BEGIN;
    DELETE FROM journal_entry_lines
    WHERE journal_entry_id IN (
        '11111111-1111-1111-1111-111111111111',
        '22222222-2222-2222-2222-222222222222',
        '33333333-3333-3333-3333-333333333333'
    );

    DELETE FROM journal_entries
    WHERE id IN (
        '11111111-1111-1111-1111-111111111111',
        '22222222-2222-2222-2222-222222222222',
        '33333333-3333-3333-3333-333333333333'
    );
    COMMIT;

    \echo '测试数据已清理'
\else
    \echo '保留测试数据'
\endif

-- ============================================================================
-- 完成
-- ============================================================================

\echo ''
\echo '=========================================='
\echo '  测试完成'
\echo '=========================================='
\echo ''
\echo '所有测试已完成！'
\echo ''
\echo '下一步:'
\echo '  1. 查看文档: cat UMSKZ_DOCUMENTATION_INDEX.md'
\echo '  2. 学习查询: cat UMSKZ_DATABASE_VIEWS_GUIDE.md'
\echo '  3. 开始使用: 参考 UMSKZ_QUICK_REFERENCE.md'
\echo ''
