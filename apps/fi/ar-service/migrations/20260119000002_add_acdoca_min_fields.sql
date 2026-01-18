-- Add minimal ACDOCA-aligned fields for AR
-- Date: 2026-01-19

-- =============================================================================
-- Invoices: add minimal fields for UJ alignment
-- =============================================================================
ALTER TABLE invoices
ADD COLUMN IF NOT EXISTS ledger VARCHAR(2),
ADD COLUMN IF NOT EXISTS special_gl_indicator VARCHAR(1),
ADD COLUMN IF NOT EXISTS payment_terms VARCHAR(10),
ADD COLUMN IF NOT EXISTS payment_method VARCHAR(1),
ADD COLUMN IF NOT EXISTS payment_block VARCHAR(1),
ADD COLUMN IF NOT EXISTS transaction_type VARCHAR(4),
ADD COLUMN IF NOT EXISTS reference_transaction_type VARCHAR(5),
ADD COLUMN IF NOT EXISTS baseline_date DATE;

COMMENT ON COLUMN invoices.ledger IS 'RLDNR 分类账';
COMMENT ON COLUMN invoices.special_gl_indicator IS 'UMSKZ 特殊总账标识';
COMMENT ON COLUMN invoices.payment_terms IS 'ZTERM 付款条件';
COMMENT ON COLUMN invoices.payment_method IS 'ZLSCH 付款方式';
COMMENT ON COLUMN invoices.payment_block IS 'ZLSPR 付款冻结';
COMMENT ON COLUMN invoices.transaction_type IS 'VRGNG 业务交易类型';
COMMENT ON COLUMN invoices.reference_transaction_type IS 'AWTYP 参考交易类型';
COMMENT ON COLUMN invoices.baseline_date IS 'ZFBDT 基准日期';

-- =============================================================================
-- Open items: add minimal fields for UJ alignment
-- =============================================================================
ALTER TABLE open_items
ADD COLUMN IF NOT EXISTS ledger VARCHAR(2),
ADD COLUMN IF NOT EXISTS special_gl_indicator VARCHAR(1),
ADD COLUMN IF NOT EXISTS payment_method VARCHAR(1),
ADD COLUMN IF NOT EXISTS payment_terms VARCHAR(10),
ADD COLUMN IF NOT EXISTS dunning_block VARCHAR(1),
ADD COLUMN IF NOT EXISTS dunning_level INT,
ADD COLUMN IF NOT EXISTS transaction_type VARCHAR(4),
ADD COLUMN IF NOT EXISTS reference_transaction_type VARCHAR(5),
ADD COLUMN IF NOT EXISTS baseline_date DATE;

COMMENT ON COLUMN open_items.ledger IS 'RLDNR 分类账';
COMMENT ON COLUMN open_items.special_gl_indicator IS 'UMSKZ 特殊总账标识';
COMMENT ON COLUMN open_items.payment_method IS 'ZLSCH 付款方式';
COMMENT ON COLUMN open_items.payment_terms IS 'ZTERM 付款条件';
COMMENT ON COLUMN open_items.dunning_block IS 'MANST 催款冻结';
COMMENT ON COLUMN open_items.dunning_level IS 'MADAT 催款级别';
COMMENT ON COLUMN open_items.transaction_type IS 'VRGNG 业务交易类型';
COMMENT ON COLUMN open_items.reference_transaction_type IS 'AWTYP 参考交易类型';
COMMENT ON COLUMN open_items.baseline_date IS 'ZFBDT 基准日期';
