-- 添加发票拒绝原因字段
-- 用于存储发票审批被拒绝时的原因说明
ALTER TABLE invoices
ADD COLUMN IF NOT EXISTS rejection_reason TEXT;

COMMENT ON COLUMN invoices.rejection_reason IS '拒绝原因 - 发票审批被拒绝时的原因说明';

-- 按拒绝状态查询的索引
CREATE INDEX IF NOT EXISTS idx_invoices_rejected
ON invoices(status) WHERE status = 'REJECTED';
