-- GL Service Database Schema
-- 凭证头表 (Journal Entry Header)
CREATE TABLE IF NOT EXISTS journal_entries (
    id UUID PRIMARY KEY,
    document_number VARCHAR(20),
    company_code VARCHAR(4) NOT NULL,
    fiscal_year INT NOT NULL,
    fiscal_period INT NOT NULL,
    posting_date DATE NOT NULL,
    document_date DATE NOT NULL,
    entry_date DATE NOT NULL DEFAULT CURRENT_DATE,
    document_type VARCHAR(2) NOT NULL DEFAULT 'SA',
    currency VARCHAR(3) NOT NULL DEFAULT 'CNY',
    reference VARCHAR(50),
    header_text VARCHAR(255),
    status VARCHAR(20) NOT NULL DEFAULT 'DRAFT',
    
    -- 审计字段
    created_by VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    posted_by VARCHAR(50),
    posted_at TIMESTAMPTZ,
    reversed_by VARCHAR(50),
    reversed_at TIMESTAMPTZ,
    reversal_document_id UUID,
    
    -- 多租户
    tenant_id VARCHAR(50),
    
    CONSTRAINT uq_document_number UNIQUE (company_code, fiscal_year, document_number)
);

-- 凭证行表 (Journal Entry Line Items)
CREATE TABLE IF NOT EXISTS journal_entry_lines (
    id UUID PRIMARY KEY,
    journal_entry_id UUID NOT NULL REFERENCES journal_entries(id) ON DELETE CASCADE,
    line_number INT NOT NULL,
    
    -- 科目信息
    account_id VARCHAR(10) NOT NULL,
    account_type VARCHAR(20),
    
    -- 金额
    debit_credit CHAR(1) NOT NULL CHECK (debit_credit IN ('D', 'C')),
    amount DECIMAL(15, 2) NOT NULL,
    local_amount DECIMAL(15, 2) NOT NULL,
    local_currency VARCHAR(3) NOT NULL DEFAULT 'CNY',
    exchange_rate DECIMAL(10, 6) DEFAULT 1.000000,
    
    -- 成本对象
    cost_center VARCHAR(10),
    profit_center VARCHAR(10),
    segment VARCHAR(10),
    
    -- 业务伙伴
    business_partner VARCHAR(10),
    
    -- 描述
    line_text VARCHAR(255),
    assignment VARCHAR(50),
    
    -- 时间戳
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT uq_journal_line UNIQUE (journal_entry_id, line_number)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_journal_entries_company_date ON journal_entries(company_code, posting_date);
CREATE INDEX IF NOT EXISTS idx_journal_entries_status ON journal_entries(status);
CREATE INDEX IF NOT EXISTS idx_journal_entry_lines_account ON journal_entry_lines(account_id);
CREATE INDEX IF NOT EXISTS idx_journal_entry_lines_cost_center ON journal_entry_lines(cost_center);
