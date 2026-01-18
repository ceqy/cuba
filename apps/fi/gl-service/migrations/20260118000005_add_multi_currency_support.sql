-- Migration: Add Multi-Currency Support (RHCUR, RKCUR, RTCUR, OSL, VSL)
-- Description: 添加多币种支持字段，用于多币种报表、集团合并和管理会计
-- Date: 2026-01-18

-- ============================================================================
-- 1. 添加多币种字段到凭证头表
-- ============================================================================
ALTER TABLE journal_entries
ADD COLUMN IF NOT EXISTS local_currency VARCHAR(3) DEFAULT 'CNY',
ADD COLUMN IF NOT EXISTS group_currency VARCHAR(3) DEFAULT '',
ADD COLUMN IF NOT EXISTS target_currency VARCHAR(3) DEFAULT '';

COMMENT ON COLUMN journal_entries.local_currency IS '本位币 (RHCUR) - 公司记账本位币';
COMMENT ON COLUMN journal_entries.group_currency IS '集团货币 (RKCUR) - 用于集团合并报表';
COMMENT ON COLUMN journal_entries.target_currency IS '目标货币 (RTCUR) - 用于管理报表';

-- ============================================================================
-- 2. 添加多币种金额字段到凭证行表
-- ============================================================================
ALTER TABLE journal_entry_lines
ADD COLUMN IF NOT EXISTS amount_in_object_currency DECIMAL(15, 2),
ADD COLUMN IF NOT EXISTS object_currency VARCHAR(3) DEFAULT '',
ADD COLUMN IF NOT EXISTS amount_in_profit_center_currency DECIMAL(15, 2),
ADD COLUMN IF NOT EXISTS profit_center_currency VARCHAR(3) DEFAULT '',
ADD COLUMN IF NOT EXISTS amount_in_group_currency DECIMAL(15, 2),
ADD COLUMN IF NOT EXISTS group_currency VARCHAR(3) DEFAULT '';

COMMENT ON COLUMN journal_entry_lines.amount_in_object_currency IS '对象货币金额 (OSL) - 用于成本对象核算';
COMMENT ON COLUMN journal_entry_lines.object_currency IS '对象货币代码';
COMMENT ON COLUMN journal_entry_lines.amount_in_profit_center_currency IS '利润中心货币金额 (VSL) - 用于利润中心报表';
COMMENT ON COLUMN journal_entry_lines.profit_center_currency IS '利润中心货币代码';
COMMENT ON COLUMN journal_entry_lines.amount_in_group_currency IS '集团货币金额 - 用于集团合并';
COMMENT ON COLUMN journal_entry_lines.group_currency IS '集团货币代码';

-- ============================================================================
-- 3. 创建索引（性能优化）
-- ============================================================================
-- 按本位币查询的索引
CREATE INDEX IF NOT EXISTS idx_journal_entries_local_currency
ON journal_entries(local_currency)
WHERE local_currency IS NOT NULL AND local_currency != '';

-- 按集团货币查询的索引
CREATE INDEX IF NOT EXISTS idx_journal_entries_group_currency
ON journal_entries(group_currency)
WHERE group_currency IS NOT NULL AND group_currency != '';

-- ============================================================================
-- 4. 更新现有数据（向后兼容）
-- ============================================================================
-- 将所有现有凭证的本位币设置为 CNY（如果为空）
UPDATE journal_entries
SET local_currency = 'CNY'
WHERE local_currency IS NULL OR local_currency = '';

-- 将所有现有行项目的货币字段设置为空（使用默认值）
UPDATE journal_entry_lines
SET object_currency = '',
    profit_center_currency = '',
    group_currency = ''
WHERE object_currency IS NULL
   OR profit_center_currency IS NULL
   OR group_currency IS NULL;

-- ============================================================================
-- 5. 创建多币种汇总视图（按货币汇总）
-- ============================================================================
CREATE OR REPLACE VIEW v_multi_currency_summary AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    je.currency as document_currency,
    je.local_currency,
    je.group_currency,
    je.target_currency,
    COUNT(DISTINCT je.id) as document_count,
    COUNT(jel.id) as line_item_count,
    -- 凭证货币金额
    SUM(jel.amount) as total_document_currency_amount,
    -- 本位币金额
    SUM(jel.local_amount) as total_local_currency_amount,
    -- 集团货币金额
    SUM(jel.amount_in_group_currency) as total_group_currency_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    je.currency,
    je.local_currency,
    je.group_currency,
    je.target_currency
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period;

COMMENT ON VIEW v_multi_currency_summary IS '多币种汇总视图 - 按货币类型汇总金额';

-- ============================================================================
-- 6. 创建外币交易分析视图
-- ============================================================================
CREATE OR REPLACE VIEW v_foreign_currency_transactions AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    je.document_number,
    je.posting_date,
    je.currency as document_currency,
    je.local_currency,
    je.exchange_rate,
    jel.line_number,
    jel.account_id,
    jel.amount as document_currency_amount,
    jel.local_amount as local_currency_amount,
    -- 计算汇兑差异
    CASE
        WHEN je.currency != je.local_currency
        THEN jel.local_amount - (jel.amount * CAST(je.exchange_rate AS DECIMAL(10,6)))
        ELSE 0
    END as exchange_difference,
    je.status
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.currency != je.local_currency
  AND je.status = 'POSTED'
ORDER BY
    je.company_code,
    je.posting_date DESC;

COMMENT ON VIEW v_foreign_currency_transactions IS '外币交易分析视图 - 显示外币交易和汇兑差异';

-- ============================================================================
-- 7. 创建集团货币合并视图
-- ============================================================================
CREATE OR REPLACE VIEW v_group_currency_consolidation AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    je.group_currency,
    jel.account_id,
    jel.debit_credit,
    COUNT(*) as transaction_count,
    SUM(jel.amount_in_group_currency) as total_group_currency_amount,
    SUM(CASE WHEN jel.debit_credit = 'D' THEN jel.amount_in_group_currency ELSE 0 END) as debit_amount,
    SUM(CASE WHEN jel.debit_credit = 'C' THEN jel.amount_in_group_currency ELSE 0 END) as credit_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND je.group_currency IS NOT NULL
  AND je.group_currency != ''
  AND jel.amount_in_group_currency IS NOT NULL
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    je.group_currency,
    jel.account_id,
    jel.debit_credit
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    je.group_currency;

COMMENT ON VIEW v_group_currency_consolidation IS '集团货币合并视图 - 用于集团合并报表';

-- ============================================================================
-- 8. 创建利润中心货币报表视图
-- ============================================================================
CREATE OR REPLACE VIEW v_profit_center_currency_report AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.profit_center,
    jel.profit_center_currency,
    jel.account_id,
    COUNT(*) as transaction_count,
    SUM(jel.amount_in_profit_center_currency) as total_profit_center_currency_amount,
    SUM(CASE WHEN jel.debit_credit = 'D' THEN jel.amount_in_profit_center_currency ELSE 0 END) as debit_amount,
    SUM(CASE WHEN jel.debit_credit = 'C' THEN jel.amount_in_profit_center_currency ELSE 0 END) as credit_amount,
    -- 计算净额
    SUM(CASE
        WHEN jel.debit_credit = 'D' THEN jel.amount_in_profit_center_currency
        ELSE -jel.amount_in_profit_center_currency
    END) as net_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.profit_center IS NOT NULL
  AND jel.profit_center != ''
  AND jel.amount_in_profit_center_currency IS NOT NULL
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.profit_center,
    jel.profit_center_currency,
    jel.account_id
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.profit_center;

COMMENT ON VIEW v_profit_center_currency_report IS '利润中心货币报表视图 - 按利润中心货币核算';

-- ============================================================================
-- 9. 创建对象货币核算视图
-- ============================================================================
CREATE OR REPLACE VIEW v_object_currency_accounting AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.cost_center,
    jel.object_currency,
    jel.account_id,
    COUNT(*) as transaction_count,
    SUM(jel.amount_in_object_currency) as total_object_currency_amount,
    SUM(CASE WHEN jel.debit_credit = 'D' THEN jel.amount_in_object_currency ELSE 0 END) as debit_amount,
    SUM(CASE WHEN jel.debit_credit = 'C' THEN jel.amount_in_object_currency ELSE 0 END) as credit_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.cost_center IS NOT NULL
  AND jel.cost_center != ''
  AND jel.amount_in_object_currency IS NOT NULL
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.cost_center,
    jel.object_currency,
    jel.account_id
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.cost_center;

COMMENT ON VIEW v_object_currency_accounting IS '对象货币核算视图 - 用于成本对象核算';

-- ============================================================================
-- 10. 创建货币使用统计视图
-- ============================================================================
CREATE OR REPLACE VIEW v_currency_usage_statistics AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.currency as currency_code,
    'Document Currency' as currency_type,
    COUNT(DISTINCT je.id) as document_count,
    COUNT(jel.id) as line_item_count,
    SUM(jel.amount) as total_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
GROUP BY je.company_code, je.fiscal_year, je.currency

UNION ALL

SELECT
    je.company_code,
    je.fiscal_year,
    je.local_currency as currency_code,
    'Local Currency' as currency_type,
    COUNT(DISTINCT je.id) as document_count,
    COUNT(jel.id) as line_item_count,
    SUM(jel.local_amount) as total_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
GROUP BY je.company_code, je.fiscal_year, je.local_currency

UNION ALL

SELECT
    je.company_code,
    je.fiscal_year,
    je.group_currency as currency_code,
    'Group Currency' as currency_type,
    COUNT(DISTINCT je.id) as document_count,
    COUNT(jel.id) as line_item_count,
    SUM(jel.amount_in_group_currency) as total_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND je.group_currency IS NOT NULL
  AND je.group_currency != ''
GROUP BY je.company_code, je.fiscal_year, je.group_currency

ORDER BY company_code, fiscal_year, currency_type, currency_code;

COMMENT ON VIEW v_currency_usage_statistics IS '货币使用统计视图 - 统计各类货币的使用情况';

-- ============================================================================
-- 11. 创建汇率差异分析视图
-- ============================================================================
CREATE OR REPLACE VIEW v_exchange_rate_variance_analysis AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    je.currency as foreign_currency,
    je.local_currency,
    je.exchange_rate,
    COUNT(DISTINCT je.id) as document_count,
    SUM(jel.amount) as total_foreign_currency_amount,
    SUM(jel.local_amount) as total_local_currency_amount,
    -- 计算理论本位币金额
    SUM(jel.amount * CAST(COALESCE(NULLIF(je.exchange_rate, ''), '1.0') AS DECIMAL(10,6))) as calculated_local_amount,
    -- 计算汇兑差异
    SUM(jel.local_amount) - SUM(jel.amount * CAST(COALESCE(NULLIF(je.exchange_rate, ''), '1.0') AS DECIMAL(10,6))) as total_exchange_variance,
    -- 计算平均汇率
    AVG(CAST(COALESCE(NULLIF(je.exchange_rate, ''), '1.0') AS DECIMAL(10,6))) as average_exchange_rate
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND je.currency != je.local_currency
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    je.currency,
    je.local_currency,
    je.exchange_rate
HAVING SUM(jel.local_amount) - SUM(jel.amount * CAST(COALESCE(NULLIF(je.exchange_rate, ''), '1.0') AS DECIMAL(10,6))) != 0
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period;

COMMENT ON VIEW v_exchange_rate_variance_analysis IS '汇率差异分析视图 - 分析汇兑损益';

-- ============================================================================
-- 12. 创建多币种数据质量检查视图
-- ============================================================================
CREATE OR REPLACE VIEW v_multi_currency_data_quality AS
SELECT
    'missing_exchange_rate' as issue_type,
    '外币交易缺少汇率' as issue_description,
    je.company_code,
    je.document_number,
    je.posting_date,
    je.currency as document_currency,
    je.local_currency,
    je.exchange_rate
FROM journal_entries je
WHERE je.status = 'POSTED'
  AND je.currency != je.local_currency
  AND (je.exchange_rate IS NULL OR je.exchange_rate = '' OR je.exchange_rate = '0')

UNION ALL

SELECT
    'missing_group_currency_amount' as issue_type,
    '集团合并需要集团货币金额但缺失' as issue_description,
    je.company_code,
    je.document_number,
    je.posting_date,
    je.currency,
    je.group_currency,
    NULL as exchange_rate
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND je.group_currency IS NOT NULL
  AND je.group_currency != ''
  AND jel.amount_in_group_currency IS NULL

UNION ALL

SELECT
    'inconsistent_currency' as issue_type,
    '货币代码不一致' as issue_description,
    je.company_code,
    je.document_number,
    je.posting_date,
    je.currency,
    je.local_currency,
    NULL as exchange_rate
FROM journal_entries je
WHERE je.status = 'POSTED'
  AND je.currency = je.local_currency
  AND je.exchange_rate IS NOT NULL
  AND je.exchange_rate != ''
  AND je.exchange_rate != '1.0'
  AND je.exchange_rate != '1'

ORDER BY posting_date DESC;

COMMENT ON VIEW v_multi_currency_data_quality IS '多币种数据质量检查视图 - 识别货币相关的数据问题';

-- ============================================================================
-- 13. 创建统计信息收集函数
-- ============================================================================
CREATE OR REPLACE FUNCTION analyze_multi_currency_tables()
RETURNS void AS $$
BEGIN
    -- 收集统计信息以优化查询性能
    ANALYZE journal_entries;
    ANALYZE journal_entry_lines;
    RAISE NOTICE '已收集多币种相关表的统计信息';
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION analyze_multi_currency_tables() IS '收集多币种相关表的统计信息以优化查询性能';

-- ============================================================================
-- 14. 执行初始统计信息收集
-- ============================================================================
SELECT analyze_multi_currency_tables();

-- ============================================================================
-- 15. 迁移完成信息
-- ============================================================================
DO $$
BEGIN
    RAISE NOTICE '========================================';
    RAISE NOTICE '多币种支持 (RHCUR, RKCUR, RTCUR, OSL, VSL) 迁移完成！';
    RAISE NOTICE '========================================';
    RAISE NOTICE '已创建:';
    RAISE NOTICE '  - 凭证头: 3 个货币字段 (local_currency, group_currency, target_currency)';
    RAISE NOTICE '  - 凭证行: 6 个金额/货币字段';
    RAISE NOTICE '  - 2 个索引: 货币查询优化';
    RAISE NOTICE '  - 8 个视图: 多币种分析视图';
    RAISE NOTICE '  - 1 个函数: 维护工具';
    RAISE NOTICE '========================================';
    RAISE NOTICE '支持的功能:';
    RAISE NOTICE '  - 多币种财务报表';
    RAISE NOTICE '  - 集团合并报表（集团货币）';
    RAISE NOTICE '  - 利润中心报表（利润中心货币）';
    RAISE NOTICE '  - 成本对象核算（对象货币）';
    RAISE NOTICE '  - 汇兑损益分析';
    RAISE NOTICE '========================================';
END $$;
