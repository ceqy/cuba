-- Migration: Add Organizational Dimensions (RFAREA, RBUSA, KOKRS)
-- Description: 添加财务范围、业务范围、控制范围字段，用于合并报表、段报告和管理会计
-- Date: 2026-01-18

-- ============================================================================
-- 1. 添加组织维度字段到凭证行表
-- ============================================================================
ALTER TABLE journal_entry_lines
ADD COLUMN IF NOT EXISTS financial_area VARCHAR(4) DEFAULT '',
ADD COLUMN IF NOT EXISTS business_area VARCHAR(4) DEFAULT '',
ADD COLUMN IF NOT EXISTS controlling_area VARCHAR(4) DEFAULT '';

COMMENT ON COLUMN journal_entry_lines.financial_area IS '财务范围 (RFAREA) - 用于合并报表编制';
COMMENT ON COLUMN journal_entry_lines.business_area IS '业务范围 (RBUSA) - 用于段报告和业务分析';
COMMENT ON COLUMN journal_entry_lines.controlling_area IS '控制范围 (KOKRS) - 用于管理会计和成本控制';

-- ============================================================================
-- 2. 创建索引（性能优化）
-- ============================================================================
-- 按财务范围查询的索引（用于合并报表）
CREATE INDEX IF NOT EXISTS idx_journal_lines_financial_area
ON journal_entry_lines(financial_area)
WHERE financial_area IS NOT NULL AND financial_area != '';

-- 按业务范围查询的索引（用于段报告）
CREATE INDEX IF NOT EXISTS idx_journal_lines_business_area
ON journal_entry_lines(business_area)
WHERE business_area IS NOT NULL AND business_area != '';

-- 按控制范围查询的索引（用于管理会计）
CREATE INDEX IF NOT EXISTS idx_journal_lines_controlling_area
ON journal_entry_lines(controlling_area)
WHERE controlling_area IS NOT NULL AND controlling_area != '';

-- 复合索引：支持按公司、财务范围、业务范围查询（用于合并报表）
CREATE INDEX IF NOT EXISTS idx_journal_lines_company_areas
ON journal_entry_lines(financial_area, business_area)
WHERE financial_area IS NOT NULL AND financial_area != '';

-- ============================================================================
-- 3. 更新现有数据（向后兼容）
-- ============================================================================
-- 将所有现有行项目的组织维度字段设置为空（使用默认值）
UPDATE journal_entry_lines
SET financial_area = '',
    business_area = '',
    controlling_area = ''
WHERE financial_area IS NULL
   OR business_area IS NULL
   OR controlling_area IS NULL;

-- ============================================================================
-- 4. 创建财务范围汇总视图（用于合并报表）
-- ============================================================================
CREATE OR REPLACE VIEW v_financial_area_summary AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.financial_area,
    jel.account_id,
    jel.debit_credit,
    COUNT(*) as transaction_count,
    SUM(jel.local_amount) as total_local_amount,
    jel.local_currency as currency
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.financial_area IS NOT NULL
  AND jel.financial_area != ''
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.financial_area,
    jel.account_id,
    jel.debit_credit,
    jel.local_currency
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.financial_area;

COMMENT ON VIEW v_financial_area_summary IS '财务范围汇总视图 - 用于合并报表编制';

-- ============================================================================
-- 5. 创建业务范围汇总视图（用于段报告）
-- ============================================================================
CREATE OR REPLACE VIEW v_business_area_summary AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.business_area,
    jel.account_id,
    jel.debit_credit,
    COUNT(*) as transaction_count,
    SUM(jel.local_amount) as total_local_amount,
    SUM(CASE WHEN jel.debit_credit = 'D' THEN jel.local_amount ELSE 0 END) as total_debit,
    SUM(CASE WHEN jel.debit_credit = 'C' THEN jel.local_amount ELSE 0 END) as total_credit,
    jel.local_currency as currency
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.business_area IS NOT NULL
  AND jel.business_area != ''
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.business_area,
    jel.account_id,
    jel.debit_credit,
    jel.local_currency
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.business_area;

COMMENT ON VIEW v_business_area_summary IS '业务范围汇总视图 - 用于段报告和业务分析';

-- ============================================================================
-- 6. 创建控制范围汇总视图（用于管理会计）
-- ============================================================================
CREATE OR REPLACE VIEW v_controlling_area_summary AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.controlling_area,
    jel.cost_center,
    jel.profit_center,
    jel.account_id,
    COUNT(*) as transaction_count,
    SUM(jel.local_amount) as total_local_amount,
    SUM(CASE WHEN jel.debit_credit = 'D' THEN jel.local_amount ELSE 0 END) as total_debit,
    SUM(CASE WHEN jel.debit_credit = 'C' THEN jel.local_amount ELSE 0 END) as total_credit,
    jel.local_currency as currency
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.controlling_area IS NOT NULL
  AND jel.controlling_area != ''
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.controlling_area,
    jel.cost_center,
    jel.profit_center,
    jel.account_id,
    jel.local_currency
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.controlling_area;

COMMENT ON VIEW v_controlling_area_summary IS '控制范围汇总视图 - 用于管理会计和成本控制';

-- ============================================================================
-- 7. 创建合并报表准备视图（财务范围 + 业务范围）
-- ============================================================================
CREATE OR REPLACE VIEW v_consolidation_report AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.financial_area,
    jel.business_area,
    jel.account_id,
    jel.debit_credit,
    COUNT(*) as transaction_count,
    SUM(jel.local_amount) as total_amount,
    SUM(CASE WHEN jel.debit_credit = 'D' THEN jel.local_amount ELSE 0 END) as debit_amount,
    SUM(CASE WHEN jel.debit_credit = 'C' THEN jel.local_amount ELSE 0 END) as credit_amount,
    jel.local_currency as currency
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND (jel.financial_area IS NOT NULL AND jel.financial_area != '')
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.financial_area,
    jel.business_area,
    jel.account_id,
    jel.debit_credit,
    jel.local_currency
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.financial_area,
    jel.business_area;

COMMENT ON VIEW v_consolidation_report IS '合并报表准备视图 - 按财务范围和业务范围汇总';

-- ============================================================================
-- 8. 创建段报告视图（业务范围详细分析）
-- ============================================================================
CREATE OR REPLACE VIEW v_segment_reporting AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.business_area,
    jel.account_id,
    -- 收入
    SUM(CASE
        WHEN jel.account_id LIKE '4%' AND jel.debit_credit = 'C'
        THEN jel.local_amount
        ELSE 0
    END) as revenue,
    -- 成本
    SUM(CASE
        WHEN jel.account_id LIKE '5%' AND jel.debit_credit = 'D'
        THEN jel.local_amount
        ELSE 0
    END) as cost,
    -- 费用
    SUM(CASE
        WHEN jel.account_id LIKE '6%' AND jel.debit_credit = 'D'
        THEN jel.local_amount
        ELSE 0
    END) as expense,
    -- 资产
    SUM(CASE
        WHEN jel.account_id LIKE '1%' AND jel.debit_credit = 'D'
        THEN jel.local_amount
        WHEN jel.account_id LIKE '1%' AND jel.debit_credit = 'C'
        THEN -jel.local_amount
        ELSE 0
    END) as assets,
    -- 负债
    SUM(CASE
        WHEN jel.account_id LIKE '2%' AND jel.debit_credit = 'C'
        THEN jel.local_amount
        WHEN jel.account_id LIKE '2%' AND jel.debit_credit = 'D'
        THEN -jel.local_amount
        ELSE 0
    END) as liabilities,
    jel.local_currency as currency
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.business_area IS NOT NULL
  AND jel.business_area != ''
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.business_area,
    jel.account_id,
    jel.local_currency
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.business_area;

COMMENT ON VIEW v_segment_reporting IS '段报告视图 - 按业务范围分析收入、成本、费用、资产、负债';

-- ============================================================================
-- 9. 创建管理会计报表视图（控制范围 + 成本中心）
-- ============================================================================
CREATE OR REPLACE VIEW v_management_accounting_report AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.controlling_area,
    jel.cost_center,
    jel.profit_center,
    jel.account_id,
    COUNT(*) as transaction_count,
    SUM(jel.local_amount) as total_amount,
    SUM(CASE WHEN jel.debit_credit = 'D' THEN jel.local_amount ELSE 0 END) as debit_amount,
    SUM(CASE WHEN jel.debit_credit = 'C' THEN jel.local_amount ELSE 0 END) as credit_amount,
    -- 计算净额（借方 - 贷方）
    SUM(CASE
        WHEN jel.debit_credit = 'D' THEN jel.local_amount
        ELSE -jel.local_amount
    END) as net_amount,
    jel.local_currency as currency
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.controlling_area IS NOT NULL
  AND jel.controlling_area != ''
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.controlling_area,
    jel.cost_center,
    jel.profit_center,
    jel.account_id,
    jel.local_currency
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.controlling_area,
    jel.cost_center;

COMMENT ON VIEW v_management_accounting_report IS '管理会计报表视图 - 按控制范围、成本中心、利润中心汇总';

-- ============================================================================
-- 10. 创建跨维度分析视图（财务范围 + 业务范围 + 控制范围）
-- ============================================================================
CREATE OR REPLACE VIEW v_cross_dimensional_analysis AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.financial_area,
    jel.business_area,
    jel.controlling_area,
    jel.cost_center,
    jel.profit_center,
    jel.account_id,
    COUNT(*) as transaction_count,
    SUM(jel.local_amount) as total_amount,
    jel.local_currency as currency
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND (
      (jel.financial_area IS NOT NULL AND jel.financial_area != '')
      OR (jel.business_area IS NOT NULL AND jel.business_area != '')
      OR (jel.controlling_area IS NOT NULL AND jel.controlling_area != '')
  )
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.financial_area,
    jel.business_area,
    jel.controlling_area,
    jel.cost_center,
    jel.profit_center,
    jel.account_id,
    jel.local_currency
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period;

COMMENT ON VIEW v_cross_dimensional_analysis IS '跨维度分析视图 - 综合分析财务范围、业务范围、控制范围';

-- ============================================================================
-- 11. 创建组织维度覆盖率分析视图
-- ============================================================================
CREATE OR REPLACE VIEW v_organizational_dimension_coverage AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    COUNT(*) as total_lines,
    COUNT(CASE WHEN jel.financial_area IS NOT NULL AND jel.financial_area != '' THEN 1 END) as lines_with_financial_area,
    COUNT(CASE WHEN jel.business_area IS NOT NULL AND jel.business_area != '' THEN 1 END) as lines_with_business_area,
    COUNT(CASE WHEN jel.controlling_area IS NOT NULL AND jel.controlling_area != '' THEN 1 END) as lines_with_controlling_area,
    ROUND(
        COUNT(CASE WHEN jel.financial_area IS NOT NULL AND jel.financial_area != '' THEN 1 END)::NUMERIC /
        NULLIF(COUNT(*)::NUMERIC, 0) * 100,
        2
    ) as financial_area_coverage_percent,
    ROUND(
        COUNT(CASE WHEN jel.business_area IS NOT NULL AND jel.business_area != '' THEN 1 END)::NUMERIC /
        NULLIF(COUNT(*)::NUMERIC, 0) * 100,
        2
    ) as business_area_coverage_percent,
    ROUND(
        COUNT(CASE WHEN jel.controlling_area IS NOT NULL AND jel.controlling_area != '' THEN 1 END)::NUMERIC /
        NULLIF(COUNT(*)::NUMERIC, 0) * 100,
        2
    ) as controlling_area_coverage_percent
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period;

COMMENT ON VIEW v_organizational_dimension_coverage IS '组织维度覆盖率分析视图 - 评估各维度字段的填充率';

-- ============================================================================
-- 12. 创建数据质量检查视图
-- ============================================================================
CREATE OR REPLACE VIEW v_organizational_dimension_quality AS
SELECT
    'missing_financial_area_for_consolidation' as issue_type,
    '合并报表需要财务范围但缺失' as issue_description,
    je.company_code,
    je.document_number,
    je.posting_date,
    jel.line_number,
    jel.account_id,
    jel.local_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND (jel.financial_area IS NULL OR jel.financial_area = '')
  AND jel.account_id LIKE '1%'  -- 资产类科目通常需要财务范围

UNION ALL

SELECT
    'missing_business_area_for_segment' as issue_type,
    '段报告需要业务范围但缺失' as issue_description,
    je.company_code,
    je.document_number,
    je.posting_date,
    jel.line_number,
    jel.account_id,
    jel.local_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND (jel.business_area IS NULL OR jel.business_area = '')
  AND jel.account_id LIKE '4%'  -- 收入类科目通常需要业务范围

UNION ALL

SELECT
    'missing_controlling_area_for_cost' as issue_type,
    '成本核算需要控制范围但缺失' as issue_description,
    je.company_code,
    je.document_number,
    je.posting_date,
    jel.line_number,
    jel.account_id,
    jel.local_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND (jel.controlling_area IS NULL OR jel.controlling_area = '')
  AND jel.cost_center IS NOT NULL
  AND jel.cost_center != ''

ORDER BY posting_date DESC;

COMMENT ON VIEW v_organizational_dimension_quality IS '组织维度数据质量检查视图 - 识别缺失的维度信息';

-- ============================================================================
-- 13. 创建统计信息收集函数
-- ============================================================================
CREATE OR REPLACE FUNCTION analyze_organizational_dimension_tables()
RETURNS void AS $$
BEGIN
    -- 收集统计信息以优化查询性能
    ANALYZE journal_entry_lines;
    RAISE NOTICE '已收集 journal_entry_lines 表的统计信息';
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION analyze_organizational_dimension_tables() IS '收集组织维度相关表的统计信息以优化查询性能';

-- ============================================================================
-- 14. 执行初始统计信息收集
-- ============================================================================
SELECT analyze_organizational_dimension_tables();

-- ============================================================================
-- 15. 迁移完成信息
-- ============================================================================
DO $$
BEGIN
    RAISE NOTICE '========================================';
    RAISE NOTICE '组织维度字段 (RFAREA, RBUSA, KOKRS) 迁移完成！';
    RAISE NOTICE '========================================';
    RAISE NOTICE '已创建:';
    RAISE NOTICE '  - 3 个字段: financial_area, business_area, controlling_area';
    RAISE NOTICE '  - 4 个索引: 单列索引 + 复合索引';
    RAISE NOTICE '  - 9 个视图: 业务分析视图';
    RAISE NOTICE '  - 1 个函数: 维护工具';
    RAISE NOTICE '========================================';
    RAISE NOTICE '支持的功能:';
    RAISE NOTICE '  - 合并报表编制（财务范围）';
    RAISE NOTICE '  - 段报告和业务分析（业务范围）';
    RAISE NOTICE '  - 管理会计和成本控制（控制范围）';
    RAISE NOTICE '========================================';
END $$;
