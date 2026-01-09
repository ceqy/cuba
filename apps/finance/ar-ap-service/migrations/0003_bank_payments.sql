-- ============================================================================
-- AR/AP Service: Bank Accounts and Payment Methods
-- 描述: 银行账户、付款方式、银行交易记录
-- ============================================================================

-- 银行主数据
CREATE TABLE bank_master (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bank_key VARCHAR(15) NOT NULL UNIQUE,
    bank_name VARCHAR(60) NOT NULL,
    bank_country VARCHAR(3) NOT NULL,
    swift_code VARCHAR(11),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 伙伴银行账户
CREATE TABLE partner_bank_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id VARCHAR(18) NOT NULL UNIQUE,
    partner_id VARCHAR(10) NOT NULL,
    bank_key VARCHAR(15) NOT NULL REFERENCES bank_master(bank_key),
    account_number VARCHAR(35) NOT NULL,
    iban VARCHAR(34),
    account_holder VARCHAR(60),
    currency VARCHAR(3) NOT NULL,
    is_primary BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 付款运行记录
CREATE TABLE payment_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id VARCHAR(20) NOT NULL UNIQUE,
    company_code VARCHAR(4) NOT NULL,
    run_date DATE NOT NULL,
    payment_method VARCHAR(20) NOT NULL, -- WIRE, CHECK, ACH, CARD
    total_payments INT NOT NULL DEFAULT 0,
    total_amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    status VARCHAR(20) DEFAULT 'CREATED', -- CREATED, PROCESSING, COMPLETED, FAILED
    created_by UUID NOT NULL,
    executed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 付款运行明细
CREATE TABLE payment_run_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id UUID NOT NULL REFERENCES payment_runs(id) ON DELETE CASCADE,
    partner_id VARCHAR(10) NOT NULL,
    bank_account_id VARCHAR(18),
    payment_amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    reference VARCHAR(50),
    status VARCHAR(20) DEFAULT 'PENDING', -- PENDING, SUCCESS, FAILED
    error_message VARCHAR(200),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 预付款/预收款表
CREATE TABLE advance_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    advance_id VARCHAR(20) NOT NULL UNIQUE,
    company_code VARCHAR(4) NOT NULL,
    partner_id VARCHAR(10) NOT NULL,
    account_type account_type NOT NULL,
    posting_date DATE NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    remaining_amount DECIMAL(15,2) NOT NULL,
    gl_account VARCHAR(10),
    reference VARCHAR(50),
    status VARCHAR(20) DEFAULT 'ACTIVE', -- ACTIVE, FULLY_APPLIED, CANCELLED
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 预付款使用记录
CREATE TABLE advance_payment_applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    advance_id UUID NOT NULL REFERENCES advance_payments(id),
    open_item_id UUID NOT NULL REFERENCES open_items(id),
    applied_amount DECIMAL(15,2) NOT NULL,
    application_date DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_partner_bank_accounts_partner ON partner_bank_accounts(partner_id);
CREATE INDEX idx_partner_bank_accounts_primary ON partner_bank_accounts(partner_id, is_primary) WHERE is_primary = TRUE;
CREATE INDEX idx_payment_runs_date ON payment_runs(run_date DESC);
CREATE INDEX idx_payment_runs_status ON payment_runs(status);
CREATE INDEX idx_payment_run_items_partner ON payment_run_items(partner_id);
CREATE INDEX idx_advance_payments_partner ON advance_payments(partner_id, status);
CREATE INDEX idx_advance_applications_advance ON advance_payment_applications(advance_id);

-- 触发器：自动更新预付款余额
CREATE OR REPLACE FUNCTION update_advance_payment_balance()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE advance_payments
    SET remaining_amount = remaining_amount - NEW.applied_amount,
        status = CASE 
            WHEN (remaining_amount - NEW.applied_amount) <= 0 THEN 'FULLY_APPLIED'
            ELSE 'ACTIVE'
        END
    WHERE id = NEW.advance_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tr_update_advance_balance
    AFTER INSERT ON advance_payment_applications
    FOR EACH ROW
    EXECUTE FUNCTION update_advance_payment_balance();
