-- Migration: Add Account Assignment Field (KTOSL)
-- Description: 添加科目分配字段，用于自动科目确定
-- Date: 2026-01-18

-- ============================================================================
-- 1. 添加科目分配字段到凭证行表
-- ============================================================================
ALTER TABLE journal_entry_lines
ADD COLUMN IF NOT EXISTS account_assignment VARCHAR(10);

COMMENT ON COLUMN journal_entry_lines.account_assignment IS '科目分配 (KTOSL) - 用于自动科目确定，如：GBB(存货)、ERL(收入)、AUF(费用)、MWS(进项税)、VST(销项税)';

-- ============================================================================
-- 2. 创建索引（性能优化）
-- ============================================================================
-- 按科目分配查询的索引（用于自动科目确定）
CREATE INDEX IF NOT EXISTS idx_jel_account_assignment
    ON journal_entry_lines(account_assignment)
    WHERE account_assignment IS NOT NULL AND account_assignment != '';

-- ============================================================================
-- 3. 更新现有数据（向后兼容）
-- ============================================================================
-- 将所有现有行项目的科目分配字段设置为空（使用默认值）
UPDATE journal_entry_lines
SET account_assignment = ''
WHERE account_assignment IS NULL;

-- ============================================================================
-- 4. 验证数据
-- ============================================================================
-- 验证字段已添加
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'journal_entry_lines'
        AND column_name = 'account_assignment'
    ) THEN
        RAISE EXCEPTION 'Migration failed: account_assignment column not added';
    END IF;
END $$;
