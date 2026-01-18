-- 发票状态历史表
-- 记录发票每次状态变更，支持完整审计追踪

CREATE TABLE IF NOT EXISTS invoice_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,

    -- 状态变更
    from_status VARCHAR(20),              -- 原状态（首次创建为 NULL）
    to_status VARCHAR(20) NOT NULL,       -- 新状态

    -- 变更原因
    reason TEXT,                          -- 变更原因（拒绝原因、冲销原因等）
    action_type VARCHAR(20) NOT NULL,     -- 操作类型: CREATE/SUBMIT/APPROVE/REJECT/CLEAR/REVERSE/REOPEN

    -- 操作人
    changed_by VARCHAR(50),               -- 操作人ID
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 扩展
    remarks TEXT,                         -- 备注
    metadata JSONB                        -- 扩展元数据
);

-- 按发票ID查询历史
CREATE INDEX IF NOT EXISTS idx_invoice_status_history_invoice
ON invoice_status_history(invoice_id);

-- 按时间排序
CREATE INDEX IF NOT EXISTS idx_invoice_status_history_time
ON invoice_status_history(changed_at DESC);

-- 按操作类型查询
CREATE INDEX IF NOT EXISTS idx_invoice_status_history_action
ON invoice_status_history(action_type);

COMMENT ON TABLE invoice_status_history IS '发票状态变更历史 - 完整审计追踪';
COMMENT ON COLUMN invoice_status_history.action_type IS '操作类型: CREATE-创建, SUBMIT-提交, APPROVE-审批通过, REJECT-拒绝, CLEAR-清账, REVERSE-冲销, REOPEN-重新打开';
