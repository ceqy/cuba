-- Migration: Add Special GL Indicator (UMSKZ)
-- Description: 添加特殊总账标识字段，用于区分票据、预付款、预收款等特殊业务类型
-- Date: 2026-01-18

-- ============================================================================
-- 0. 添加前置依赖列（清账相关字段）
-- ============================================================================
ALTER TABLE journal_entry_lines
ADD COLUMN IF NOT EXISTS clearing_document VARCHAR(20),
ADD COLUMN IF NOT EXISTS clearing_date DATE,
ADD COLUMN IF NOT EXISTS currency VARCHAR(3) NOT NULL DEFAULT 'CNY';

COMMENT ON COLUMN journal_entry_lines.clearing_document IS '清账凭证号';
COMMENT ON COLUMN journal_entry_lines.clearing_date IS '清账日期';
COMMENT ON COLUMN journal_entry_lines.currency IS '交易货币';

-- ============================================================================
-- 1. 添加特殊总账标识字段到凭证行表
-- ============================================================================
ALTER TABLE journal_entry_lines
ADD COLUMN IF NOT EXISTS special_gl_indicator VARCHAR(1) DEFAULT '';

COMMENT ON COLUMN journal_entry_lines.special_gl_indicator IS '特殊总账标识 (UMSKZ): A=票据, F=预付款, V=预收款, W=票据贴现, 空=普通业务';

-- ============================================================================
-- 2. 创建索引（性能优化）
-- ============================================================================
-- 按特殊总账标识查询的索引（用于报表和分类查询）
CREATE INDEX IF NOT EXISTS idx_journal_entry_lines_special_gl
ON journal_entry_lines(special_gl_indicator)
WHERE special_gl_indicator IS NOT NULL AND special_gl_indicator != '';

-- 复合索引：支持按公司、科目、特殊总账标识查询
CREATE INDEX IF NOT EXISTS idx_journal_lines_account_special_gl
ON journal_entry_lines(account_id, special_gl_indicator)
WHERE special_gl_indicator IS NOT NULL AND special_gl_indicator != '';

-- ============================================================================
-- 3. 添加约束（数据完整性）
-- ============================================================================
-- 确保 special_gl_indicator 字段只包含有效值
ALTER TABLE journal_entry_lines
ADD CONSTRAINT chk_special_gl_indicator
CHECK (
    special_gl_indicator = '' OR
    special_gl_indicator IN ('A', 'F', 'V', 'W')
);

-- ============================================================================
-- 4. 创建特殊总账业务视图（便于查询和报表）
-- ============================================================================
CREATE OR REPLACE VIEW v_special_gl_items AS
SELECT
    je.company_code,
    je.document_number,
    je.fiscal_year,
    je.fiscal_period,
    je.document_date,
    je.posting_date,
    jel.line_number,
    jel.account_id,
    jel.business_partner,
    jel.special_gl_indicator,
    CASE jel.special_gl_indicator
        WHEN 'A' THEN '票据 (Bills of Exchange)'
        WHEN 'F' THEN '预付款 (Down Payment)'
        WHEN 'V' THEN '预收款 (Advance Payment)'
        WHEN 'W' THEN '票据贴现 (Bill of Exchange Discount)'
        ELSE '普通业务'
    END as special_gl_description,
    jel.amount,
    jel.local_amount,
    jel.currency,
    jel.debit_credit,
    jel.clearing_document,
    jel.clearing_date,
    CASE
        WHEN jel.clearing_document IS NOT NULL THEN 'CLEARED'
        ELSE 'OPEN'
    END as clearing_status,
    je.status as document_status
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE jel.special_gl_indicator IS NOT NULL
  AND jel.special_gl_indicator != '';

COMMENT ON VIEW v_special_gl_items IS '特殊总账项目视图 - 用于查询票据、预付款、预收款等特殊业务';

-- ============================================================================
-- 5. 创建特殊总账汇总视图（用于报表）
-- ============================================================================
CREATE OR REPLACE VIEW v_special_gl_summary AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.special_gl_indicator,
    CASE jel.special_gl_indicator
        WHEN 'A' THEN '票据 (Bills of Exchange)'
        WHEN 'F' THEN '预付款 (Down Payment)'
        WHEN 'V' THEN '预收款 (Advance Payment)'
        WHEN 'W' THEN '票据贴现 (Bill of Exchange Discount)'
        ELSE '普通业务'
    END as special_gl_description,
    jel.account_id,
    jel.debit_credit,
    COUNT(*) as transaction_count,
    SUM(jel.amount) as total_amount,
    SUM(jel.local_amount) as total_local_amount,
    SUM(CASE WHEN jel.clearing_document IS NULL THEN jel.local_amount ELSE 0 END) as open_amount,
    SUM(CASE WHEN jel.clearing_document IS NOT NULL THEN jel.local_amount ELSE 0 END) as cleared_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.special_gl_indicator IS NOT NULL
  AND jel.special_gl_indicator != ''
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.special_gl_indicator,
    jel.account_id,
    jel.debit_credit;

COMMENT ON VIEW v_special_gl_summary IS '特殊总账汇总视图 - 按类型、科目、期间汇总特殊业务';

-- ============================================================================
-- 6. 更新现有数据（向后兼容）
-- ============================================================================
-- 将所有现有凭证行的特殊总账标识设置为空（普通业务）
UPDATE journal_entry_lines
SET special_gl_indicator = ''
WHERE special_gl_indicator IS NULL;

-- ============================================================================
-- 7. 创建预付款余额视图（用于资产负债表）
-- ============================================================================
CREATE OR REPLACE VIEW v_down_payment_balance AS
SELECT
    je.company_code,
    jel.business_partner as vendor_code,
    jel.account_id,
    COUNT(*) as transaction_count,
    SUM(CASE
        WHEN jel.debit_credit = 'D' AND jel.clearing_document IS NULL
        THEN jel.local_amount
        ELSE 0
    END) as open_debit_balance,
    SUM(CASE
        WHEN jel.debit_credit = 'C' AND jel.clearing_document IS NULL
        THEN jel.local_amount
        ELSE 0
    END) as open_credit_balance,
    SUM(CASE
        WHEN jel.clearing_document IS NULL
        THEN CASE WHEN jel.debit_credit = 'D' THEN jel.local_amount ELSE -jel.local_amount END
        ELSE 0
    END) as net_open_balance,
    MAX(je.posting_date) as last_transaction_date
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.special_gl_indicator = 'F'
GROUP BY je.company_code, jel.business_partner, jel.account_id
HAVING SUM(CASE
    WHEN jel.clearing_document IS NULL
    THEN CASE WHEN jel.debit_credit = 'D' THEN jel.local_amount ELSE -jel.local_amount END
    ELSE 0
END) != 0;

COMMENT ON VIEW v_down_payment_balance IS '预付款余额视图 - 显示未清预付款余额（用于资产负债表）';

-- ============================================================================
-- 8. 创建预收款余额视图（用于资产负债表）
-- ============================================================================
CREATE OR REPLACE VIEW v_advance_payment_balance AS
SELECT
    je.company_code,
    jel.business_partner as customer_code,
    jel.account_id,
    COUNT(*) as transaction_count,
    SUM(CASE
        WHEN jel.debit_credit = 'D' AND jel.clearing_document IS NULL
        THEN jel.local_amount
        ELSE 0
    END) as open_debit_balance,
    SUM(CASE
        WHEN jel.debit_credit = 'C' AND jel.clearing_document IS NULL
        THEN jel.local_amount
        ELSE 0
    END) as open_credit_balance,
    SUM(CASE
        WHEN jel.clearing_document IS NULL
        THEN CASE WHEN jel.debit_credit = 'D' THEN jel.local_amount ELSE -jel.local_amount END
        ELSE 0
    END) as net_open_balance,
    MAX(je.posting_date) as last_transaction_date
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.special_gl_indicator = 'V'
GROUP BY je.company_code, jel.business_partner, jel.account_id
HAVING SUM(CASE
    WHEN jel.clearing_document IS NULL
    THEN CASE WHEN jel.debit_credit = 'D' THEN jel.local_amount ELSE -jel.local_amount END
    ELSE 0
END) != 0;

COMMENT ON VIEW v_advance_payment_balance IS '预收款余额视图 - 显示未清预收款余额（用于资产负债表）';

-- ============================================================================
-- 9. 创建票据到期分析视图
-- ============================================================================
CREATE OR REPLACE VIEW v_bill_maturity_analysis AS
SELECT
    je.company_code,
    je.document_number,
    je.posting_date,
    jel.business_partner,
    jel.account_id,
    jel.local_amount,
    jel.currency,
    jel.clearing_date,
    jel.clearing_document,
    CASE
        WHEN jel.clearing_document IS NOT NULL THEN '已清账'
        WHEN jel.clearing_date IS NULL THEN '未设置到期日'
        WHEN jel.clearing_date < CURRENT_DATE THEN '已到期未清'
        WHEN jel.clearing_date <= CURRENT_DATE + INTERVAL '30 days' THEN '30天内到期'
        WHEN jel.clearing_date <= CURRENT_DATE + INTERVAL '90 days' THEN '90天内到期'
        ELSE '90天后到期'
    END as maturity_status,
    CASE
        WHEN jel.clearing_document IS NULL AND jel.clearing_date IS NOT NULL
        THEN jel.clearing_date - CURRENT_DATE
        ELSE NULL
    END as days_to_maturity
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.special_gl_indicator = 'A'
ORDER BY jel.clearing_date NULLS LAST;

COMMENT ON VIEW v_bill_maturity_analysis IS '票据到期分析视图 - 用于票据管理和风险控制';

-- ============================================================================
-- 10. 创建特殊总账账龄分析视图
-- ============================================================================
CREATE OR REPLACE VIEW v_special_gl_aging AS
SELECT
    je.company_code,
    jel.special_gl_indicator,
    CASE jel.special_gl_indicator
        WHEN 'A' THEN '票据'
        WHEN 'F' THEN '预付款'
        WHEN 'V' THEN '预收款'
        WHEN 'W' THEN '票据贴现'
    END as special_gl_type,
    jel.business_partner,
    jel.account_id,
    -- 账龄分段
    SUM(CASE
        WHEN jel.clearing_document IS NULL
             AND CURRENT_DATE - je.posting_date <= 30
        THEN jel.local_amount
        ELSE 0
    END) as aging_0_30_days,
    SUM(CASE
        WHEN jel.clearing_document IS NULL
             AND CURRENT_DATE - je.posting_date > 30
             AND CURRENT_DATE - je.posting_date <= 60
        THEN jel.local_amount
        ELSE 0
    END) as aging_31_60_days,
    SUM(CASE
        WHEN jel.clearing_document IS NULL
             AND CURRENT_DATE - je.posting_date > 60
             AND CURRENT_DATE - je.posting_date <= 90
        THEN jel.local_amount
        ELSE 0
    END) as aging_61_90_days,
    SUM(CASE
        WHEN jel.clearing_document IS NULL
             AND CURRENT_DATE - je.posting_date > 90
             AND CURRENT_DATE - je.posting_date <= 180
        THEN jel.local_amount
        ELSE 0
    END) as aging_91_180_days,
    SUM(CASE
        WHEN jel.clearing_document IS NULL
             AND CURRENT_DATE - je.posting_date > 180
        THEN jel.local_amount
        ELSE 0
    END) as aging_over_180_days,
    SUM(CASE
        WHEN jel.clearing_document IS NULL
        THEN jel.local_amount
        ELSE 0
    END) as total_open_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.special_gl_indicator IN ('A', 'F', 'V', 'W')
GROUP BY
    je.company_code,
    jel.special_gl_indicator,
    jel.business_partner,
    jel.account_id
HAVING SUM(CASE WHEN jel.clearing_document IS NULL THEN jel.local_amount ELSE 0 END) != 0;

COMMENT ON VIEW v_special_gl_aging IS '特殊总账账龄分析视图 - 按账龄段统计未清项目';

-- ============================================================================
-- 11. 创建特殊总账月度趋势视图
-- ============================================================================
CREATE OR REPLACE VIEW v_special_gl_monthly_trend AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.special_gl_indicator,
    CASE jel.special_gl_indicator
        WHEN 'A' THEN '票据'
        WHEN 'F' THEN '预付款'
        WHEN 'V' THEN '预收款'
        WHEN 'W' THEN '票据贴现'
    END as special_gl_type,
    -- 本期发生额
    COUNT(*) as transaction_count,
    SUM(jel.local_amount) as period_amount,
    -- 借方发生额
    SUM(CASE WHEN jel.debit_credit = 'D' THEN jel.local_amount ELSE 0 END) as debit_amount,
    -- 贷方发生额
    SUM(CASE WHEN jel.debit_credit = 'C' THEN jel.local_amount ELSE 0 END) as credit_amount,
    -- 本期清账金额
    SUM(CASE
        WHEN jel.clearing_document IS NOT NULL
             AND DATE_TRUNC('month', jel.clearing_date) = DATE_TRUNC('month', je.posting_date)
        THEN jel.local_amount
        ELSE 0
    END) as cleared_in_period,
    -- 期末未清金额（需要通过累计计算）
    SUM(CASE WHEN jel.clearing_document IS NULL THEN jel.local_amount ELSE 0 END) as open_at_period_end
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.special_gl_indicator IN ('A', 'F', 'V', 'W')
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.special_gl_indicator
ORDER BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.special_gl_indicator;

COMMENT ON VIEW v_special_gl_monthly_trend IS '特殊总账月度趋势视图 - 用于分析特殊业务的月度变化趋势';

-- ============================================================================
-- 12. 创建特殊总账清账效率分析视图
-- ============================================================================
CREATE OR REPLACE VIEW v_special_gl_clearing_efficiency AS
SELECT
    je.company_code,
    je.fiscal_year,
    jel.special_gl_indicator,
    CASE jel.special_gl_indicator
        WHEN 'A' THEN '票据'
        WHEN 'F' THEN '预付款'
        WHEN 'V' THEN '预收款'
        WHEN 'W' THEN '票据贴现'
    END as special_gl_type,
    -- 总笔数
    COUNT(*) as total_count,
    -- 已清账笔数
    COUNT(CASE WHEN jel.clearing_document IS NOT NULL THEN 1 END) as cleared_count,
    -- 未清账笔数
    COUNT(CASE WHEN jel.clearing_document IS NULL THEN 1 END) as open_count,
    -- 清账率
    ROUND(
        COUNT(CASE WHEN jel.clearing_document IS NOT NULL THEN 1 END)::NUMERIC /
        NULLIF(COUNT(*)::NUMERIC, 0) * 100,
        2
    ) as clearing_rate_percent,
    -- 平均清账天数
    ROUND(
        AVG(CASE
            WHEN jel.clearing_document IS NOT NULL
            THEN (jel.clearing_date - je.posting_date)::NUMERIC
            ELSE NULL
        END),
        1
    ) as avg_clearing_days,
    -- 总金额
    SUM(jel.local_amount) as total_amount,
    -- 已清金额
    SUM(CASE WHEN jel.clearing_document IS NOT NULL THEN jel.local_amount ELSE 0 END) as cleared_amount,
    -- 未清金额
    SUM(CASE WHEN jel.clearing_document IS NULL THEN jel.local_amount ELSE 0 END) as open_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.special_gl_indicator IN ('A', 'F', 'V', 'W')
GROUP BY
    je.company_code,
    je.fiscal_year,
    jel.special_gl_indicator
ORDER BY
    je.company_code,
    je.fiscal_year,
    jel.special_gl_indicator;

COMMENT ON VIEW v_special_gl_clearing_efficiency IS '特殊总账清账效率分析视图 - 用于评估清账效率和资金周转';

-- ============================================================================
-- 13. 创建业务伙伴特殊总账汇总视图
-- ============================================================================
CREATE OR REPLACE VIEW v_business_partner_special_gl AS
SELECT
    je.company_code,
    jel.business_partner,
    jel.special_gl_indicator,
    CASE jel.special_gl_indicator
        WHEN 'A' THEN '票据'
        WHEN 'F' THEN '预付款'
        WHEN 'V' THEN '预收款'
        WHEN 'W' THEN '票据贴现'
    END as special_gl_type,
    COUNT(*) as transaction_count,
    SUM(jel.local_amount) as total_amount,
    SUM(CASE WHEN jel.clearing_document IS NULL THEN jel.local_amount ELSE 0 END) as open_amount,
    SUM(CASE WHEN jel.clearing_document IS NOT NULL THEN jel.local_amount ELSE 0 END) as cleared_amount,
    MIN(je.posting_date) as first_transaction_date,
    MAX(je.posting_date) as last_transaction_date,
    MAX(CASE WHEN jel.clearing_document IS NULL THEN je.posting_date END) as last_open_transaction_date
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.special_gl_indicator IN ('A', 'F', 'V', 'W')
  AND jel.business_partner IS NOT NULL
  AND jel.business_partner != ''
GROUP BY
    je.company_code,
    jel.business_partner,
    jel.special_gl_indicator
ORDER BY
    je.company_code,
    jel.business_partner,
    jel.special_gl_indicator;

COMMENT ON VIEW v_business_partner_special_gl IS '业务伙伴特殊总账汇总视图 - 按供应商/客户汇总特殊业务';

-- ============================================================================
-- 14. 创建特殊总账风险预警视图
-- ============================================================================
CREATE OR REPLACE VIEW v_special_gl_risk_alert AS
SELECT
    je.company_code,
    je.document_number,
    je.posting_date,
    jel.special_gl_indicator,
    CASE jel.special_gl_indicator
        WHEN 'A' THEN '票据'
        WHEN 'F' THEN '预付款'
        WHEN 'V' THEN '预收款'
        WHEN 'W' THEN '票据贴现'
    END as special_gl_type,
    jel.business_partner,
    jel.account_id,
    jel.local_amount,
    CURRENT_DATE - je.posting_date as days_outstanding,
    CASE
        -- 票据风险
        WHEN jel.special_gl_indicator = 'A' AND jel.clearing_document IS NULL
             AND jel.clearing_date < CURRENT_DATE
        THEN '票据已到期未清'
        -- 预付款风险
        WHEN jel.special_gl_indicator = 'F' AND jel.clearing_document IS NULL
             AND CURRENT_DATE - je.posting_date > 180
        THEN '预付款超过180天未清'
        -- 预收款风险
        WHEN jel.special_gl_indicator = 'V' AND jel.clearing_document IS NULL
             AND CURRENT_DATE - je.posting_date > 365
        THEN '预收款超过1年未清'
        -- 一般风险
        WHEN jel.clearing_document IS NULL AND CURRENT_DATE - je.posting_date > 90
        THEN '超过90天未清'
        ELSE NULL
    END as risk_alert,
    CASE
        WHEN jel.special_gl_indicator = 'A' AND jel.clearing_document IS NULL
             AND jel.clearing_date < CURRENT_DATE
        THEN 'HIGH'
        WHEN jel.clearing_document IS NULL AND CURRENT_DATE - je.posting_date > 180
        THEN 'HIGH'
        WHEN jel.clearing_document IS NULL AND CURRENT_DATE - je.posting_date > 90
        THEN 'MEDIUM'
        WHEN jel.clearing_document IS NULL AND CURRENT_DATE - je.posting_date > 30
        THEN 'LOW'
        ELSE NULL
    END as risk_level
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.special_gl_indicator IN ('A', 'F', 'V', 'W')
  AND jel.clearing_document IS NULL
  AND (
      -- 票据已到期
      (jel.special_gl_indicator = 'A' AND jel.clearing_date < CURRENT_DATE)
      -- 或超过30天未清
      OR (CURRENT_DATE - je.posting_date > 30)
  )
ORDER BY
    CASE
        WHEN jel.special_gl_indicator = 'A' AND jel.clearing_date < CURRENT_DATE THEN 1
        WHEN CURRENT_DATE - je.posting_date > 180 THEN 2
        WHEN CURRENT_DATE - je.posting_date > 90 THEN 3
        ELSE 4
    END,
    jel.local_amount DESC;

COMMENT ON VIEW v_special_gl_risk_alert IS '特殊总账风险预警视图 - 识别需要关注的异常项目';

-- ============================================================================
-- 15. 创建性能优化的物化视图（可选）
-- ============================================================================
-- 注意：物化视图需要定期刷新，适合大数据量场景

-- 特殊总账余额物化视图（每日刷新）
CREATE MATERIALIZED VIEW IF NOT EXISTS mv_special_gl_balance AS
SELECT
    je.company_code,
    jel.special_gl_indicator,
    jel.account_id,
    jel.business_partner,
    COUNT(*) as transaction_count,
    SUM(CASE WHEN jel.clearing_document IS NULL THEN jel.local_amount ELSE 0 END) as open_balance,
    MAX(je.posting_date) as last_posting_date,
    CURRENT_DATE as snapshot_date
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
  AND jel.special_gl_indicator IN ('A', 'F', 'V', 'W')
GROUP BY
    je.company_code,
    jel.special_gl_indicator,
    jel.account_id,
    jel.business_partner
HAVING SUM(CASE WHEN jel.clearing_document IS NULL THEN jel.local_amount ELSE 0 END) != 0;

-- 创建物化视图索引
CREATE INDEX IF NOT EXISTS idx_mv_special_gl_balance_company
ON mv_special_gl_balance(company_code, special_gl_indicator);

CREATE INDEX IF NOT EXISTS idx_mv_special_gl_balance_partner
ON mv_special_gl_balance(business_partner)
WHERE business_partner IS NOT NULL;

COMMENT ON MATERIALIZED VIEW mv_special_gl_balance IS '特殊总账余额物化视图 - 用于快速查询余额（需定期刷新）';

-- ============================================================================
-- 16. 创建刷新物化视图的函数
-- ============================================================================
CREATE OR REPLACE FUNCTION refresh_special_gl_materialized_views()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY mv_special_gl_balance;
    RAISE NOTICE '特殊总账物化视图已刷新: %', NOW();
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION refresh_special_gl_materialized_views() IS '刷新特殊总账相关的物化视图';

-- ============================================================================
-- 17. 创建数据质量检查视图
-- ============================================================================
CREATE OR REPLACE VIEW v_special_gl_data_quality AS
SELECT
    'missing_business_partner' as issue_type,
    '特殊总账项目缺少业务伙伴' as issue_description,
    je.company_code,
    je.document_number,
    je.posting_date,
    jel.special_gl_indicator,
    jel.line_number,
    jel.local_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE jel.special_gl_indicator IN ('A', 'F', 'V', 'W')
  AND (jel.business_partner IS NULL OR jel.business_partner = '')

UNION ALL

SELECT
    'bill_missing_maturity_date' as issue_type,
    '票据缺少到期日' as issue_description,
    je.company_code,
    je.document_number,
    je.posting_date,
    jel.special_gl_indicator,
    jel.line_number,
    jel.local_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE jel.special_gl_indicator = 'A'
  AND jel.clearing_date IS NULL
  AND jel.clearing_document IS NULL

UNION ALL

SELECT
    'long_outstanding' as issue_type,
    '长期未清项目（超过1年）' as issue_description,
    je.company_code,
    je.document_number,
    je.posting_date,
    jel.special_gl_indicator,
    jel.line_number,
    jel.local_amount
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE jel.special_gl_indicator IN ('A', 'F', 'V', 'W')
  AND jel.clearing_document IS NULL
  AND CURRENT_DATE - je.posting_date > 365

ORDER BY posting_date DESC;

COMMENT ON VIEW v_special_gl_data_quality IS '特殊总账数据质量检查视图 - 识别数据质量问题';

-- ============================================================================
-- 18. 创建统计信息收集函数
-- ============================================================================
CREATE OR REPLACE FUNCTION analyze_special_gl_tables()
RETURNS void AS $$
BEGIN
    -- 收集统计信息以优化查询性能
    ANALYZE journal_entry_lines;
    RAISE NOTICE '已收集 journal_entry_lines 表的统计信息';
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION analyze_special_gl_tables() IS '收集特殊总账相关表的统计信息以优化查询性能';

-- ============================================================================
-- 19. 执行初始统计信息收集
-- ============================================================================
SELECT analyze_special_gl_tables();

-- ============================================================================
-- 20. 迁移完成信息
-- ============================================================================
DO $$
BEGIN
    RAISE NOTICE '========================================';
    RAISE NOTICE '特殊总账标识 (UMSKZ) 迁移完成！';
    RAISE NOTICE '========================================';
    RAISE NOTICE '已创建:';
    RAISE NOTICE '  - 1 个字段: special_gl_indicator';
    RAISE NOTICE '  - 2 个索引: 单列索引 + 复合索引';
    RAISE NOTICE '  - 1 个约束: 数据完整性检查';
    RAISE NOTICE '  - 13 个视图: 业务分析视图';
    RAISE NOTICE '  - 1 个物化视图: 性能优化';
    RAISE NOTICE '  - 2 个函数: 维护工具';
    RAISE NOTICE '========================================';
    RAISE NOTICE '支持的特殊总账类型:';
    RAISE NOTICE '  A = 票据 (Bills of Exchange)';
    RAISE NOTICE '  F = 预付款 (Down Payment)';
    RAISE NOTICE '  V = 预收款 (Advance Payment)';
    RAISE NOTICE '  W = 票据贴现 (Bill Discount)';
    RAISE NOTICE '========================================';
END $$;
