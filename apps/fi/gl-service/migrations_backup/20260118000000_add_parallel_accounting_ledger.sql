-- Migration: Add Parallel Accounting Support (RLDNR)
-- Description: 添加并行会计支持，允许同一笔业务在多个分类账中记录
-- Date: 2026-01-18

-- ============================================================================
-- 1. 添加分类账字段到凭证头表
-- ============================================================================
ALTER TABLE journal_entries
ADD COLUMN IF NOT EXISTS ledger_group VARCHAR(4) DEFAULT '',
ADD COLUMN IF NOT EXISTS default_ledger VARCHAR(2) DEFAULT '0L';

COMMENT ON COLUMN journal_entries.ledger_group IS '分类账组 (LDGRP)';
COMMENT ON COLUMN journal_entries.default_ledger IS '默认分类账 (RLDNR: 0L=主账, 1L/2L=非主账)';

-- ============================================================================
-- 2. 添加分类账字段到凭证行表
-- ============================================================================
ALTER TABLE journal_entry_lines
ADD COLUMN IF NOT EXISTS ledger VARCHAR(2) DEFAULT '0L' NOT NULL,
ADD COLUMN IF NOT EXISTS ledger_type INT DEFAULT 1 NOT NULL,
ADD COLUMN IF NOT EXISTS ledger_amount DECIMAL(15, 2);

COMMENT ON COLUMN journal_entry_lines.ledger IS '分类账编号 (RLDNR: 0L, 1L, 2L...)';
COMMENT ON COLUMN journal_entry_lines.ledger_type IS '分类账类型 (1=主账, 2=非主账, 3=扩展账)';
COMMENT ON COLUMN journal_entry_lines.ledger_amount IS '分类账货币金额（用于不同会计准则下的金额差异）';

-- ============================================================================
-- 3. 更新现有数据（向后兼容）
-- ============================================================================
-- 将所有现有凭证设置为主分类账 (0L)
UPDATE journal_entries
SET default_ledger = '0L'
WHERE default_ledger IS NULL OR default_ledger = '';

UPDATE journal_entry_lines
SET ledger = '0L', ledger_type = 1
WHERE ledger IS NULL OR ledger = '';

-- ============================================================================
-- 4. 创建并行会计索引（性能优化）
-- ============================================================================
-- 按分类账查询凭证的索引
CREATE INDEX IF NOT EXISTS idx_journal_entries_ledger
ON journal_entries(company_code, fiscal_year, default_ledger);

-- 按分类账查询行项目的索引
CREATE INDEX IF NOT EXISTS idx_journal_entry_lines_ledger
ON journal_entry_lines(ledger, account_id);

-- 复合索引：支持按公司、年度、分类账、科目查询
CREATE INDEX IF NOT EXISTS idx_journal_lines_ledger_account
ON journal_entry_lines(journal_entry_id, ledger, account_id);

-- ============================================================================
-- 5. 添加约束（数据完整性）
-- ============================================================================
-- 确保 ledger 字段符合 SAP 命名规范
ALTER TABLE journal_entry_lines
ADD CONSTRAINT chk_ledger_format
CHECK (ledger ~ '^[0-9][A-Z]$');

-- 确保 ledger_type 在有效范围内
ALTER TABLE journal_entry_lines
ADD CONSTRAINT chk_ledger_type
CHECK (ledger_type IN (1, 2, 3));

-- ============================================================================
-- 6. 创建并行会计视图（便于查询）
-- ============================================================================
CREATE OR REPLACE VIEW v_parallel_accounting_summary AS
SELECT
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.ledger,
    jel.ledger_type,
    jel.account_id,
    jel.debit_credit,
    SUM(jel.amount) as total_amount,
    SUM(jel.local_amount) as total_local_amount,
    COUNT(*) as transaction_count
FROM journal_entries je
JOIN journal_entry_lines jel ON je.id = jel.journal_entry_id
WHERE je.status = 'POSTED'
GROUP BY
    je.company_code,
    je.fiscal_year,
    je.fiscal_period,
    jel.ledger,
    jel.ledger_type,
    jel.account_id,
    jel.debit_credit;

COMMENT ON VIEW v_parallel_accounting_summary IS '并行会计汇总视图 - 按分类账、科目、期间汇总';

-- ============================================================================
-- 7. 创建分类账余额表（可选 - 用于性能优化）
-- ============================================================================
CREATE TABLE IF NOT EXISTS ledger_balances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_code VARCHAR(4) NOT NULL,
    fiscal_year INT NOT NULL,
    fiscal_period INT NOT NULL,
    ledger VARCHAR(2) NOT NULL,
    account_id VARCHAR(10) NOT NULL,

    -- 余额
    opening_balance DECIMAL(15, 2) DEFAULT 0,
    debit_total DECIMAL(15, 2) DEFAULT 0,
    credit_total DECIMAL(15, 2) DEFAULT 0,
    closing_balance DECIMAL(15, 2) DEFAULT 0,

    -- 货币
    currency VARCHAR(3) NOT NULL DEFAULT 'CNY',

    -- 时间戳
    last_updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_ledger_balance UNIQUE (company_code, fiscal_year, fiscal_period, ledger, account_id)
);

CREATE INDEX IF NOT EXISTS idx_ledger_balances_lookup
ON ledger_balances(company_code, fiscal_year, ledger, account_id);

COMMENT ON TABLE ledger_balances IS '分类账余额表 - 用于快速查询各分类账的科目余额';

-- ============================================================================
-- 8. 创建触发器函数（自动更新余额）
-- ============================================================================
CREATE OR REPLACE FUNCTION update_ledger_balance()
RETURNS TRIGGER AS $$
BEGIN
    -- 当凭证过账时，更新对应分类账的余额
    IF NEW.status = 'POSTED' AND (OLD.status IS NULL OR OLD.status != 'POSTED') THEN
        -- 这里可以添加余额更新逻辑
        -- 实际生产环境中，建议使用异步任务处理
        NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 注释掉触发器，避免影响性能（可根据需要启用）
-- CREATE TRIGGER trg_update_ledger_balance
-- AFTER UPDATE ON journal_entries
-- FOR EACH ROW
-- EXECUTE FUNCTION update_ledger_balance();
