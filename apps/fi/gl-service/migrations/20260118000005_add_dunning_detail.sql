-- Migration: Add Dunning Detail Fields (MSCHL)
-- Description: 添加催款相关字段，用于应收/应付账款的催款管理
-- Date: 2026-01-18

-- ============================================================================
-- 1. 添加催款相关字段到凭证行表
-- ============================================================================
ALTER TABLE journal_entry_lines
ADD COLUMN IF NOT EXISTS dunning_key VARCHAR(1),
ADD COLUMN IF NOT EXISTS dunning_block VARCHAR(1),
ADD COLUMN IF NOT EXISTS last_dunning_date DATE,
ADD COLUMN IF NOT EXISTS dunning_date DATE,
ADD COLUMN IF NOT EXISTS dunning_level INT DEFAULT 0,
ADD COLUMN IF NOT EXISTS dunning_area VARCHAR(2),
ADD COLUMN IF NOT EXISTS grace_period_days INT DEFAULT 0,
ADD COLUMN IF NOT EXISTS dunning_charges DECIMAL(15, 2) DEFAULT 0,
ADD COLUMN IF NOT EXISTS dunning_clerk VARCHAR(12);

COMMENT ON COLUMN journal_entry_lines.dunning_key IS '催款码 (MSCHL) - 催款程序标识';
COMMENT ON COLUMN journal_entry_lines.dunning_block IS '催款冻结 (MANST) - 冻结原因代码';
COMMENT ON COLUMN journal_entry_lines.last_dunning_date IS '上次催款日期 (MADAT)';
COMMENT ON COLUMN journal_entry_lines.dunning_date IS '催款日期 (MANDT) - 下次催款日期';
COMMENT ON COLUMN journal_entry_lines.dunning_level IS '催款级别 (1-9，级别越高越严厉)';
COMMENT ON COLUMN journal_entry_lines.dunning_area IS '催款范围 (MAHNA) - 用于区分不同催款策略';
COMMENT ON COLUMN journal_entry_lines.grace_period_days IS '宽限期天数';
COMMENT ON COLUMN journal_entry_lines.dunning_charges IS '催款费用 - 每次催款收取的费用';
COMMENT ON COLUMN journal_entry_lines.dunning_clerk IS '催款员 - 负责催款的人员';

-- ============================================================================
-- 2. 创建催款相关索引（性能优化）
-- ============================================================================
-- 按催款日期查询（查找需要催款的项目）
CREATE INDEX IF NOT EXISTS idx_journal_lines_dunning_date
ON journal_entry_lines(dunning_date)
WHERE dunning_date IS NOT NULL
  AND dunning_block IS NULL;

-- 按催款级别查询（查找不同级别的催款项目）
CREATE INDEX IF NOT EXISTS idx_journal_lines_dunning_level
ON journal_entry_lines(dunning_level, dunning_date)
WHERE dunning_level > 0
  AND dunning_block IS NULL;

-- 按催款冻结状态查询
CREATE INDEX IF NOT EXISTS idx_journal_lines_dunning_block
ON journal_entry_lines(dunning_block)
WHERE dunning_block IS NOT NULL;

-- 按催款员查询（查找某个催款员负责的项目）
CREATE INDEX IF NOT EXISTS idx_journal_lines_dunning_clerk
ON journal_entry_lines(dunning_clerk, dunning_date)
WHERE dunning_clerk IS NOT NULL;

-- ============================================================================
-- 3. 创建催款级别枚举类型（可选）
-- ============================================================================
DO $$ BEGIN
    CREATE TYPE dunning_level_type AS ENUM (
        'LEVEL_0',  -- 无催款
        'LEVEL_1',  -- 第一次催款（友好提醒）
        'LEVEL_2',  -- 第二次催款（正式通知）
        'LEVEL_3',  -- 第三次催款（严厉警告）
        'LEVEL_4',  -- 第四次催款（最后通牒）
        'LEVEL_5',  -- 第五次催款（法律程序）
        'LEVEL_6',  -- 第六次催款
        'LEVEL_7',  -- 第七次催款
        'LEVEL_8',  -- 第八次催款
        'LEVEL_9'   -- 第九次催款（最高级别）
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- ============================================================================
-- 4. 创建催款视图（待催款项目）
-- ============================================================================
CREATE OR REPLACE VIEW v_dunning_items AS
SELECT
    je.id as journal_entry_id,
    je.document_number,
    je.company_code,
    je.fiscal_year,
    je.posting_date,
    jel.id as line_id,
    jel.line_number,
    jel.account_id,
    jel.business_partner,
    jel.amount,
    jel.local_amount,

    -- 催款信息
    jel.dunning_key,
    jel.dunning_block,
    jel.last_dunning_date,
    jel.dunning_date,
    jel.dunning_level,
    jel.dunning_area,
    jel.grace_period_days,
    jel.dunning_charges,
    jel.dunning_clerk,

    -- 计算逾期天数
    CASE
        WHEN jel.dunning_date IS NOT NULL THEN
            CURRENT_DATE - jel.dunning_date
        ELSE NULL
    END as overdue_days,

    -- 判断是否需要催款
    CASE
        WHEN jel.dunning_block IS NOT NULL THEN false  -- 已冻结
        WHEN jel.dunning_date IS NULL THEN false       -- 无催款日期
        WHEN jel.dunning_date > CURRENT_DATE THEN false -- 未到催款日期
        ELSE true                                       -- 需要催款
    END as needs_dunning,

    -- 清账状态
    jel.clearing_document,
    jel.clearing_date

FROM journal_entry_lines jel
JOIN journal_entries je ON jel.journal_entry_id = je.id
WHERE je.status = 'POSTED'
  AND jel.clearing_document IS NULL  -- 未清账
  AND jel.account_id LIKE '1%'       -- 应收账款科目（可根据实际调整）
ORDER BY jel.dunning_level DESC, jel.dunning_date ASC;

COMMENT ON VIEW v_dunning_items IS '催款项目视图 - 显示所有需要催款的未清账项目';

-- ============================================================================
-- 5. 创建催款统计视图
-- ============================================================================
CREATE OR REPLACE VIEW v_dunning_statistics AS
SELECT
    company_code,
    dunning_level,
    COUNT(*) as item_count,
    SUM(local_amount) as total_amount,
    AVG(overdue_days) as avg_overdue_days,
    MIN(dunning_date) as earliest_dunning_date,
    MAX(dunning_date) as latest_dunning_date
FROM v_dunning_items
WHERE needs_dunning = true
GROUP BY company_code, dunning_level
ORDER BY company_code, dunning_level;

COMMENT ON VIEW v_dunning_statistics IS '催款统计视图 - 按公司和催款级别汇总';

-- ============================================================================
-- 6. 创建催款历史表（可选 - 用于审计追踪）
-- ============================================================================
CREATE TABLE IF NOT EXISTS dunning_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    journal_entry_line_id UUID NOT NULL REFERENCES journal_entry_lines(id),

    -- 催款信息
    dunning_run_date DATE NOT NULL,
    dunning_level INT NOT NULL,
    dunning_key VARCHAR(1),
    dunning_area VARCHAR(2),
    dunning_charges DECIMAL(15, 2) DEFAULT 0,

    -- 客户信息
    business_partner VARCHAR(10),
    company_code VARCHAR(4) NOT NULL,

    -- 金额信息
    outstanding_amount DECIMAL(15, 2) NOT NULL,
    overdue_days INT NOT NULL,

    -- 催款结果
    dunning_letter_sent BOOLEAN DEFAULT false,
    dunning_letter_number VARCHAR(20),
    dunning_method VARCHAR(20),  -- EMAIL, MAIL, PHONE, etc.

    -- 备注
    notes TEXT,

    -- 时间戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(50),

    CONSTRAINT fk_dunning_history_line
        FOREIGN KEY (journal_entry_line_id)
        REFERENCES journal_entry_lines(id)
        ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dunning_history_line
ON dunning_history(journal_entry_line_id, dunning_run_date DESC);

CREATE INDEX IF NOT EXISTS idx_dunning_history_partner
ON dunning_history(business_partner, dunning_run_date DESC);

CREATE INDEX IF NOT EXISTS idx_dunning_history_date
ON dunning_history(dunning_run_date DESC);

COMMENT ON TABLE dunning_history IS '催款历史表 - 记录每次催款的详细信息';

-- ============================================================================
-- 7. 创建催款处理函数
-- ============================================================================
CREATE OR REPLACE FUNCTION process_dunning_run(
    p_company_code VARCHAR(4),
    p_dunning_date DATE DEFAULT CURRENT_DATE,
    p_test_run BOOLEAN DEFAULT true
)
RETURNS TABLE (
    line_id UUID,
    document_number VARCHAR(20),
    business_partner VARCHAR(10),
    current_level INT,
    new_level INT,
    outstanding_amount DECIMAL(15, 2),
    overdue_days INT,
    action VARCHAR(50)
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        jel.id as line_id,
        je.document_number,
        jel.business_partner,
        jel.dunning_level as current_level,
        CASE
            WHEN jel.dunning_level < 9 THEN jel.dunning_level + 1
            ELSE 9
        END as new_level,
        jel.local_amount as outstanding_amount,
        (p_dunning_date - jel.dunning_date) as overdue_days,
        CASE
            WHEN jel.dunning_block IS NOT NULL THEN 'BLOCKED'
            WHEN jel.dunning_date > p_dunning_date THEN 'NOT_DUE'
            WHEN jel.dunning_level >= 9 THEN 'MAX_LEVEL'
            ELSE 'ESCALATE'
        END as action
    FROM journal_entry_lines jel
    JOIN journal_entries je ON jel.journal_entry_id = je.id
    WHERE je.company_code = p_company_code
      AND je.status = 'POSTED'
      AND jel.clearing_document IS NULL
      AND jel.dunning_date IS NOT NULL
      AND jel.dunning_date <= p_dunning_date
    ORDER BY jel.dunning_level DESC, jel.dunning_date ASC;

    -- 如果不是测试运行，更新催款级别
    IF NOT p_test_run THEN
        UPDATE journal_entry_lines jel
        SET
            dunning_level = CASE
                WHEN jel.dunning_level < 9 THEN jel.dunning_level + 1
                ELSE 9
            END,
            last_dunning_date = p_dunning_date,
            dunning_date = p_dunning_date + INTERVAL '7 days'  -- 下次催款日期（可配置）
        FROM journal_entries je
        WHERE jel.journal_entry_id = je.id
          AND je.company_code = p_company_code
          AND je.status = 'POSTED'
          AND jel.clearing_document IS NULL
          AND jel.dunning_block IS NULL
          AND jel.dunning_date IS NOT NULL
          AND jel.dunning_date <= p_dunning_date;
    END IF;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION process_dunning_run IS '处理催款运行 - 识别需要催款的项目并升级催款级别';

-- ============================================================================
-- 8. 创建催款冻结/解冻函数
-- ============================================================================
CREATE OR REPLACE FUNCTION block_dunning(
    p_line_id UUID,
    p_block_reason VARCHAR(1),
    p_notes TEXT DEFAULT NULL
)
RETURNS BOOLEAN AS $$
BEGIN
    UPDATE journal_entry_lines
    SET dunning_block = p_block_reason
    WHERE id = p_line_id;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION unblock_dunning(
    p_line_id UUID
)
RETURNS BOOLEAN AS $$
BEGIN
    UPDATE journal_entry_lines
    SET dunning_block = NULL
    WHERE id = p_line_id;

    RETURN FOUND;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION block_dunning IS '冻结催款 - 临时停止对某个项目的催款';
COMMENT ON FUNCTION unblock_dunning IS '解冻催款 - 恢复对某个项目的催款';

-- ============================================================================
-- 9. 示例数据和使用说明
-- ============================================================================

-- 示例 1: 设置催款信息
-- UPDATE journal_entry_lines
-- SET
--     dunning_key = '1',
--     dunning_date = CURRENT_DATE + INTERVAL '30 days',
--     dunning_level = 0,
--     dunning_area = '01',
--     grace_period_days = 5
-- WHERE id = <line_id>;

-- 示例 2: 查询需要催款的项目
-- SELECT * FROM v_dunning_items
-- WHERE needs_dunning = true
--   AND company_code = '1000'
-- ORDER BY dunning_level DESC, overdue_days DESC;

-- 示例 3: 执行催款运行（测试模式）
-- SELECT * FROM process_dunning_run('1000', CURRENT_DATE, true);

-- 示例 4: 执行催款运行（实际运行）
-- SELECT * FROM process_dunning_run('1000', CURRENT_DATE, false);

-- 示例 5: 冻结催款
-- SELECT block_dunning(<line_id>, 'D', '客户提出争议，暂停催款');

-- 示例 6: 解冻催款
-- SELECT unblock_dunning(<line_id>);

-- 示例 7: 查询催款统计
-- SELECT * FROM v_dunning_statistics
-- WHERE company_code = '1000';

-- 示例 8: 查询催款历史
-- SELECT * FROM dunning_history
-- WHERE business_partner = 'CUST001'
-- ORDER BY dunning_run_date DESC;
