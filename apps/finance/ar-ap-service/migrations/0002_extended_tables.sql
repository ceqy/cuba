-- ============================================================================
-- AR/AP Service: Extended Tables for Advanced Features
-- 描述: 清账历史、信用管理、付款建议、账龄分析等
-- ============================================================================

-- 清账历史表
CREATE TABLE clearing_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    clearing_document VARCHAR(10) NOT NULL,
    company_code VARCHAR(4) NOT NULL,
    fiscal_year INT NOT NULL,
    clearing_date DATE NOT NULL,
    cleared_by UUID NOT NULL,
    clearing_type VARCHAR(20), -- FULL, PARTIAL, AUTOMATIC, NET
    total_amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    reference VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(company_code, clearing_document, fiscal_year)
);

-- 清账行项目关联表
CREATE TABLE clearing_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    clearing_history_id UUID NOT NULL REFERENCES clearing_history(id) ON DELETE CASCADE,
    open_item_id UUID NOT NULL REFERENCES open_items(id),
    cleared_amount DECIMAL(15,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 信用额度历史表
CREATE TABLE credit_limit_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id VARCHAR(10) NOT NULL REFERENCES customers(customer_id),
    old_limit DECIMAL(15,2),
    new_limit DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    changed_by UUID NOT NULL,
    change_reason VARCHAR(200),
    effective_date DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 信用检查记录表
CREATE TABLE credit_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id VARCHAR(10) NOT NULL REFERENCES customers(customer_id),
    check_date DATE NOT NULL,
    credit_limit DECIMAL(15,2),
    current_exposure DECIMAL(15,2),
    available_credit DECIMAL(15,2),
    check_result VARCHAR(20), -- PASS, FAIL, WARNING
    check_reason VARCHAR(200),
    checked_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 付款建议表
CREATE TABLE payment_proposals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id VARCHAR(20) NOT NULL UNIQUE,
    company_code VARCHAR(4) NOT NULL,
    proposal_date DATE NOT NULL,
    payment_date DATE NOT NULL,
    payment_method VARCHAR(20),
    total_amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    status VARCHAR(20) DEFAULT 'DRAFT', -- DRAFT, APPROVED, EXECUTED, CANCELLED
    created_by UUID NOT NULL,
    approved_by UUID,
    executed_by UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 付款建议行项目
CREATE TABLE payment_proposal_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    proposal_id UUID NOT NULL REFERENCES payment_proposals(id) ON DELETE CASCADE,
    open_item_id UUID NOT NULL REFERENCES open_items(id),
    payment_amount DECIMAL(15,2) NOT NULL,
    discount_amount DECIMAL(15,2) DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 账龄分析快照表 (定期生成，用于历史对比)
CREATE TABLE aging_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    snapshot_id VARCHAR(20) NOT NULL UNIQUE,
    company_code VARCHAR(4) NOT NULL,
    snapshot_date DATE NOT NULL,
    account_type account_type NOT NULL,
    total_current DECIMAL(15,2) DEFAULT 0,
    total_1_30_days DECIMAL(15,2) DEFAULT 0,
    total_31_60_days DECIMAL(15,2) DEFAULT 0,
    total_61_90_days DECIMAL(15,2) DEFAULT 0,
    total_over_90_days DECIMAL(15,2) DEFAULT 0,
    currency VARCHAR(3) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 催款记录表
CREATE TABLE dunning_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id VARCHAR(10) NOT NULL REFERENCES customers(customer_id),
    dunning_level INT NOT NULL, -- 1, 2, 3 (催款等级)
    dunning_date DATE NOT NULL,
    total_overdue DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    contact_method VARCHAR(20), -- EMAIL, PHONE, LETTER
    response VARCHAR(200),
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 对账单表
CREATE TABLE account_statements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    statement_id VARCHAR(20) NOT NULL UNIQUE,
    partner_id VARCHAR(10) NOT NULL,
    account_type account_type NOT NULL,
    company_code VARCHAR(4) NOT NULL,
    period_from DATE NOT NULL,
    period_to DATE NOT NULL,
    opening_balance DECIMAL(15,2) NOT NULL,
    closing_balance DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    generated_by UUID NOT NULL,
    generated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_clearing_history_date ON clearing_history(clearing_date);
CREATE INDEX idx_clearing_history_company ON clearing_history(company_code, fiscal_year);
CREATE INDEX idx_clearing_items_open_item ON clearing_items(open_item_id);
CREATE INDEX idx_credit_checks_customer ON credit_checks(customer_id, check_date DESC);
CREATE INDEX idx_payment_proposals_status ON payment_proposals(status);
CREATE INDEX idx_payment_proposals_date ON payment_proposals(payment_date);
CREATE INDEX idx_dunning_history_customer ON dunning_history(customer_id, dunning_date DESC);
CREATE INDEX idx_aging_snapshots_date ON aging_snapshots(company_code, snapshot_date DESC);
CREATE INDEX idx_account_statements_partner ON account_statements(partner_id, period_to DESC);

-- 触发器：自动更新 open_items 的清账状态
CREATE OR REPLACE FUNCTION update_open_item_clearing()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE open_items
    SET clearing_date = (SELECT clearing_date FROM clearing_history WHERE id = NEW.clearing_history_id),
        clearing_doc = (SELECT clearing_document FROM clearing_history WHERE id = NEW.clearing_history_id),
        open_amount = open_amount - NEW.cleared_amount
    WHERE id = NEW.open_item_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tr_update_clearing_status
    AFTER INSERT ON clearing_items
    FOR EACH ROW
    EXECUTE FUNCTION update_open_item_clearing();
