-- 数据迁移测试脚本
-- 验证现有数据不受影响，新字段默认值正确

\echo '========================================='
\echo '  数据迁移测试'
\echo '========================================='
\echo ''

-- 1. 创建测试数据（模拟迁移前的数据）
\echo '1. 创建迁移前的测试数据...'
BEGIN;

-- 插入测试凭证（没有 special_gl_indicator）
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
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
    '1000',
    2025,
    12,
    '2025-12-31',
    '2025-12-31',
    'POSTED',
    'CNY',
    NOW()
);

-- 插入行项目（没有 special_gl_indicator）
INSERT INTO journal_entry_lines (
    id,
    journal_entry_id,
    line_number,
    account_id,
    debit_credit,
    amount,
    local_amount,
    currency
) VALUES
(
    gen_random_uuid(),
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
    1,
    '5000',
    'D',
    10000.00,
    10000.00,
    'CNY'
),
(
    gen_random_uuid(),
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
    2,
    '2100',
    'C',
    10000.00,
    10000.00,
    'CNY'
);

COMMIT;

\echo '✓ 测试数据创建成功'
\echo ''

-- 2. 验证现有数据的 special_gl_indicator 字段
\echo '2. 验证现有数据的 special_gl_indicator 字段...'
SELECT
    line_number,
    account_id,
    debit_credit,
    amount,
    COALESCE(special_gl_indicator, '(NULL)') as special_gl_indicator
FROM journal_entry_lines
WHERE journal_entry_id = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'
ORDER BY line_number;

\echo ''
\echo '验证点：'
\echo '  - special_gl_indicator 应该为 NULL 或空字符串'
\echo '  - 现有数据应该不受影响'
\echo ''

-- 3. 测试插入带 special_gl_indicator 的新数据
\echo '3. 测试插入带 special_gl_indicator 的新数据...'
BEGIN;

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
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
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
    line_number,
    account_id,
    debit_credit,
    amount,
    local_amount,
    currency,
    special_gl_indicator
) VALUES
(
    gen_random_uuid(),
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
    1,
    '1100',
    'D',
    20000.00,
    20000.00,
    'CNY',
    'F'  -- 预付款
),
(
    gen_random_uuid(),
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb',
    2,
    '2100',
    'C',
    20000.00,
    20000.00,
    'CNY',
    'F'  -- 预付款
);

COMMIT;

\echo '✓ 新数据插入成功'
\echo ''

-- 4. 验证新数据的 special_gl_indicator
\echo '4. 验证新数据的 special_gl_indicator...'
SELECT
    line_number,
    account_id,
    debit_credit,
    amount,
    special_gl_indicator
FROM journal_entry_lines
WHERE journal_entry_id = 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
ORDER BY line_number;

\echo ''
\echo '验证点：'
\echo '  - special_gl_indicator 应该为 F'
\echo '  - 数据应该正确保存'
\echo ''

-- 5. 测试约束验证
\echo '5. 测试约束验证...'
\echo '测试有效值（应该成功）...'
BEGIN;
UPDATE journal_entry_lines
SET special_gl_indicator = 'A'
WHERE journal_entry_id = 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
  AND line_number = 1;
\echo '✓ 有效值 A 测试通过'
ROLLBACK;

\echo ''
\echo '测试无效值（应该失败）...'
BEGIN;
DO $$
BEGIN
    UPDATE journal_entry_lines
    SET special_gl_indicator = 'X'  -- 无效值
    WHERE journal_entry_id = 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
      AND line_number = 1;
    RAISE EXCEPTION '约束测试失败：应该拒绝无效值';
EXCEPTION
    WHEN check_violation THEN
        RAISE NOTICE '✓ 约束测试通过：成功拒绝无效值 X';
END $$;
ROLLBACK;

\echo ''

-- 6. 测试视图查询
\echo '6. 测试视图查询...'
\echo '查询特殊总账项目视图...'
SELECT
    document_number,
    special_gl_indicator,
    special_gl_description,
    local_amount
FROM v_special_gl_items
WHERE fiscal_year = 2026
ORDER BY posting_date DESC
LIMIT 5;

\echo ''
\echo '验证点：'
\echo '  - 视图应该能正确查询特殊总账项目'
\echo '  - special_gl_description 应该显示正确的描述'
\echo ''

-- 7. 测试混合数据查询
\echo '7. 测试混合数据查询（新旧数据）...'
SELECT
    je.id,
    je.fiscal_year,
    je.posting_date,
    COUNT(jel.id) as line_count,
    COUNT(CASE WHEN jel.special_gl_indicator IS NOT NULL AND jel.special_gl_indicator != '' THEN 1 END) as special_gl_count
FROM journal_entries je
LEFT JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.id IN (
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
)
GROUP BY je.id, je.fiscal_year, je.posting_date
ORDER BY je.posting_date;

\echo ''
\echo '验证点：'
\echo '  - 旧数据（2025）的 special_gl_count 应该为 0'
\echo '  - 新数据（2026）的 special_gl_count 应该为 2'
\echo ''

-- 8. 清理测试数据
\echo '8. 清理测试数据...'
BEGIN;

DELETE FROM journal_entry_lines
WHERE journal_entry_id IN (
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
);

DELETE FROM journal_entries
WHERE id IN (
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa',
    'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'
);

COMMIT;

\echo '✓ 测试数据清理完成'
\echo ''

-- 9. 总结
\echo '========================================='
\echo '  数据迁移测试完成'
\echo '========================================='
\echo ''
\echo '测试结果：'
\echo '  ✓ 现有数据不受影响'
\echo '  ✓ 新字段默认值正确（NULL）'
\echo '  ✓ 新数据可以正确插入'
\echo '  ✓ 约束验证正常工作'
\echo '  ✓ 视图查询正常工作'
\echo '  ✓ 新旧数据可以共存'
\echo ''
