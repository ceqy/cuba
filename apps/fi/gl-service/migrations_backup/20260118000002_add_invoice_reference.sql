-- Migration: Add Invoice Reference Fields (REBZG)
-- Description: 添加发票参考字段，用于贷项凭证追溯原始发票
-- Date: 2026-01-18

-- ============================================================================
-- 1. 添加发票参考字段到凭证行表
-- ============================================================================
ALTER TABLE journal_entry_lines
ADD COLUMN IF NOT EXISTS reference_document_number VARCHAR(20),
ADD COLUMN IF NOT EXISTS reference_fiscal_year INT,
ADD COLUMN IF NOT EXISTS reference_line_item INT,
ADD COLUMN IF NOT EXISTS reference_document_type VARCHAR(2),
ADD COLUMN IF NOT EXISTS reference_company_code VARCHAR(4);

COMMENT ON COLUMN journal_entry_lines.reference_document_number IS '参考凭证号 (REBZG) - 用于贷项凭证追溯原始发票';
COMMENT ON COLUMN journal_entry_lines.reference_fiscal_year IS '参考会计年度 (REBZJ)';
COMMENT ON COLUMN journal_entry_lines.reference_line_item IS '参考行项目号 (REBZZ)';
COMMENT ON COLUMN journal_entry_lines.reference_document_type IS '参考凭证类型 (REBZT)';
COMMENT ON COLUMN journal_entry_lines.reference_company_code IS '参考公司代码';

-- ============================================================================
-- 2. 创建发票参考索引（性能优化）
-- ============================================================================
-- 按参考凭证查询（查找所有引用某张发票的贷项凭证）
CREATE INDEX IF NOT EXISTS idx_journal_lines_reference_doc
ON journal_entry_lines(reference_company_code, reference_fiscal_year, reference_document_number)
WHERE reference_document_number IS NOT NULL;

-- 复合索引：支持按参考凭证和行项目查询
CREATE INDEX IF NOT EXISTS idx_journal_lines_reference_full
ON journal_entry_lines(reference_company_code, reference_fiscal_year, reference_document_number, reference_line_item)
WHERE reference_document_number IS NOT NULL;

-- ============================================================================
-- 3. 创建贷项凭证追溯视图
-- ============================================================================
CREATE OR REPLACE VIEW v_credit_memo_traceability AS
SELECT
    -- 贷项凭证信息
    cm_je.id as credit_memo_id,
    cm_je.document_number as credit_memo_number,
    cm_je.company_code,
    cm_je.fiscal_year as credit_memo_fiscal_year,
    cm_je.posting_date as credit_memo_date,
    cm_jel.line_number as credit_memo_line,
    cm_jel.account_id,
    cm_jel.amount as credit_memo_amount,

    -- 原始发票信息
    cm_jel.reference_document_number as original_invoice_number,
    cm_jel.reference_fiscal_year as original_fiscal_year,
    cm_jel.reference_line_item as original_line_item,
    cm_jel.reference_document_type as original_document_type,

    -- 原始发票详情（如果存在）
    orig_je.id as original_invoice_id,
    orig_je.posting_date as original_invoice_date,
    orig_jel.amount as original_invoice_amount,

    -- 差异分析
    CASE
        WHEN orig_jel.amount IS NOT NULL THEN
            cm_jel.amount - orig_jel.amount
        ELSE NULL
    END as amount_difference

FROM journal_entry_lines cm_jel
JOIN journal_entries cm_je ON cm_jel.journal_entry_id = cm_je.id
LEFT JOIN journal_entries orig_je ON
    orig_je.company_code = cm_jel.reference_company_code
    AND orig_je.fiscal_year = cm_jel.reference_fiscal_year
    AND orig_je.document_number = cm_jel.reference_document_number
LEFT JOIN journal_entry_lines orig_jel ON
    orig_jel.journal_entry_id = orig_je.id
    AND orig_jel.line_number = cm_jel.reference_line_item
WHERE cm_jel.reference_document_number IS NOT NULL;

COMMENT ON VIEW v_credit_memo_traceability IS '贷项凭证追溯视图 - 显示贷项凭证与原始发票的关联关系';

-- ============================================================================
-- 4. 创建发票参考完整性检查函数
-- ============================================================================
CREATE OR REPLACE FUNCTION check_invoice_reference_integrity()
RETURNS TABLE (
    line_id UUID,
    journal_entry_id UUID,
    document_number VARCHAR(20),
    line_number INT,
    reference_document_number VARCHAR(20),
    reference_fiscal_year INT,
    issue_type VARCHAR(50),
    issue_description TEXT
) AS $$
BEGIN
    -- 检查引用的发票是否存在
    RETURN QUERY
    SELECT
        jel.id as line_id,
        jel.journal_entry_id,
        je.document_number,
        jel.line_number,
        jel.reference_document_number,
        jel.reference_fiscal_year,
        'MISSING_REFERENCE'::VARCHAR(50) as issue_type,
        'Referenced invoice does not exist'::TEXT as issue_description
    FROM journal_entry_lines jel
    JOIN journal_entries je ON jel.journal_entry_id = je.id
    WHERE jel.reference_document_number IS NOT NULL
      AND NOT EXISTS (
          SELECT 1 FROM journal_entries ref_je
          WHERE ref_je.company_code = jel.reference_company_code
            AND ref_je.fiscal_year = jel.reference_fiscal_year
            AND ref_je.document_number = jel.reference_document_number
      );

    -- 检查引用的行项目是否存在
    RETURN QUERY
    SELECT
        jel.id as line_id,
        jel.journal_entry_id,
        je.document_number,
        jel.line_number,
        jel.reference_document_number,
        jel.reference_fiscal_year,
        'MISSING_LINE_ITEM'::VARCHAR(50) as issue_type,
        'Referenced line item does not exist'::TEXT as issue_description
    FROM journal_entry_lines jel
    JOIN journal_entries je ON jel.journal_entry_id = je.id
    WHERE jel.reference_document_number IS NOT NULL
      AND jel.reference_line_item IS NOT NULL
      AND NOT EXISTS (
          SELECT 1
          FROM journal_entries ref_je
          JOIN journal_entry_lines ref_jel ON ref_jel.journal_entry_id = ref_je.id
          WHERE ref_je.company_code = jel.reference_company_code
            AND ref_je.fiscal_year = jel.reference_fiscal_year
            AND ref_je.document_number = jel.reference_document_number
            AND ref_jel.line_number = jel.reference_line_item
      );
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION check_invoice_reference_integrity() IS '检查发票参考完整性 - 验证所有引用的发票和行项目是否存在';

-- ============================================================================
-- 5. 创建贷项凭证汇总统计表（可选 - 用于报表）
-- ============================================================================
CREATE TABLE IF NOT EXISTS credit_memo_statistics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_code VARCHAR(4) NOT NULL,
    fiscal_year INT NOT NULL,
    fiscal_period INT NOT NULL,

    -- 统计数据
    total_credit_memos INT DEFAULT 0,
    total_credit_amount DECIMAL(15, 2) DEFAULT 0,
    credit_memos_with_reference INT DEFAULT 0,
    credit_memos_without_reference INT DEFAULT 0,

    -- 按凭证类型统计
    sales_returns_count INT DEFAULT 0,
    sales_returns_amount DECIMAL(15, 2) DEFAULT 0,
    purchase_returns_count INT DEFAULT 0,
    purchase_returns_amount DECIMAL(15, 2) DEFAULT 0,

    -- 时间戳
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_credit_memo_stats UNIQUE (company_code, fiscal_year, fiscal_period)
);

CREATE INDEX IF NOT EXISTS idx_credit_memo_stats_lookup
ON credit_memo_statistics(company_code, fiscal_year, fiscal_period);

COMMENT ON TABLE credit_memo_statistics IS '贷项凭证统计表 - 用于快速查询贷项凭证汇总数据';

-- ============================================================================
-- 6. 示例数据和使用说明
-- ============================================================================

-- 示例 1: 创建原始销售发票
-- INSERT INTO journal_entries (id, document_number, company_code, fiscal_year, ...) VALUES (...);
-- INSERT INTO journal_entry_lines (id, journal_entry_id, line_number, account_id, amount, ...) VALUES (...);

-- 示例 2: 创建引用原始发票的贷项凭证
-- INSERT INTO journal_entry_lines (
--     id, journal_entry_id, line_number, account_id, amount,
--     reference_document_number, reference_fiscal_year, reference_line_item,
--     reference_document_type, reference_company_code
-- ) VALUES (
--     gen_random_uuid(), <credit_memo_je_id>, 1, '4001', -1000.00,
--     'INV-2024-001', 2024, 1, 'DR', '1000'
-- );

-- 查询示例 1: 查找某张发票的所有贷项凭证
-- SELECT * FROM v_credit_memo_traceability
-- WHERE original_invoice_number = 'INV-2024-001'
--   AND original_fiscal_year = 2024;

-- 查询示例 2: 检查发票参考完整性
-- SELECT * FROM check_invoice_reference_integrity();

-- 查询示例 3: 统计某期间的贷项凭证
-- SELECT
--     company_code,
--     fiscal_year,
--     COUNT(*) as credit_memo_count,
--     SUM(credit_memo_amount) as total_amount
-- FROM v_credit_memo_traceability
-- WHERE company_code = '1000'
--   AND credit_memo_fiscal_year = 2024
-- GROUP BY company_code, fiscal_year;
