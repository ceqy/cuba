-- Add minimal payment/bank fields for TR
-- Date: 2026-01-19

-- =============================================================================
-- Bank statements
-- =============================================================================
ALTER TABLE bank_statements
ADD COLUMN IF NOT EXISTS house_bank VARCHAR(10),
ADD COLUMN IF NOT EXISTS bank_account VARCHAR(20);

COMMENT ON COLUMN bank_statements.house_bank IS 'HBKID 内部银行账户标识';
COMMENT ON COLUMN bank_statements.bank_account IS 'BANKN 银行账号';

-- =============================================================================
-- Statement transactions
-- =============================================================================
ALTER TABLE statement_transactions
ADD COLUMN IF NOT EXISTS transaction_type VARCHAR(4);

COMMENT ON COLUMN statement_transactions.transaction_type IS 'VRGNG 业务交易类型';

-- =============================================================================
-- Payment documents
-- =============================================================================
ALTER TABLE payment_documents
ADD COLUMN IF NOT EXISTS payment_method VARCHAR(5),
ADD COLUMN IF NOT EXISTS house_bank VARCHAR(10),
ADD COLUMN IF NOT EXISTS bank_account VARCHAR(20),
ADD COLUMN IF NOT EXISTS transaction_type VARCHAR(4);

COMMENT ON COLUMN payment_documents.payment_method IS 'ZLSCH 付款方式';
COMMENT ON COLUMN payment_documents.house_bank IS 'HBKID 内部银行账户标识';
COMMENT ON COLUMN payment_documents.bank_account IS 'BANKN 银行账号';
COMMENT ON COLUMN payment_documents.transaction_type IS 'VRGNG 业务交易类型';
