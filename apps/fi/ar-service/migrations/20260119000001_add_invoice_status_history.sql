-- AR 发票状态历史表
-- 记录发票每次状态变更，支持完整审计追踪
CREATE TABLE IF NOT EXISTS invoice_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(invoice_id) ON DELETE CASCADE,

    -- 状态变更
    from_status VARCHAR(20),              -- 原状态（首次创建为 NULL）
    to_status VARCHAR(20) NOT NULL,       -- 新状态
    reason TEXT,                          -- 变更原因（拒绝原因等）
    action_type VARCHAR(20) NOT NULL,     -- 操作类型: CREATE/APPROVE/REJECT/CLEAR/REVERSE

    -- 操作人信息
    changed_by VARCHAR(50),               -- 操作人 ID
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- 扩展字段
    remarks TEXT,                         -- 备注
    metadata JSONB                        -- 扩展元数据
);

-- 发票状态历史索引
CREATE INDEX IF NOT EXISTS idx_ish_invoice_id ON invoice_status_history(invoice_id);
CREATE INDEX IF NOT EXISTS idx_ish_changed_at ON invoice_status_history(changed_at);
CREATE INDEX IF NOT EXISTS idx_ish_action_type ON invoice_status_history(action_type);

COMMENT ON TABLE invoice_status_history IS 'AR 发票状态历史表 - 记录每次状态变更的审计追踪';
