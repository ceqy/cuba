-- ============================================================================
-- GL Service: Tax and Clearing Tables
-- 描述: 税务行项目和清账凭证表
-- ============================================================================

-- ============================================================================
-- 税务行项目表 (参考: BSET)
-- ============================================================================
CREATE TABLE journal_entry_tax (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 外键
    header_id UUID NOT NULL REFERENCES journal_entry_headers(id) ON DELETE CASCADE,
    line_id UUID REFERENCES journal_entry_lines(id) ON DELETE CASCADE,
    
    -- 税务信息
    line_number INT NOT NULL,
    tax_code VARCHAR(4) NOT NULL,
    tax_rate DECIMAL(5, 2),
    tax_type VARCHAR(10), -- VST=进项税, MWS=销项税
    
    -- 税基和税额 (凭证货币)
    tax_base_amount_doc DECIMAL(15, 2),
    tax_amount_doc DECIMAL(15, 2) NOT NULL,
    
    -- 税基和税额 (本位币)
    tax_base_amount_local DECIMAL(15, 2),
    tax_amount_local DECIMAL(15, 2),
    
    -- 借贷标识
    debit_credit_indicator VARCHAR(1) NOT NULL,
    
    -- 税务管辖
    tax_country VARCHAR(3),
    tax_jurisdiction VARCHAR(20),
    
    -- 是否自动税
    is_auto_tax BOOLEAN DEFAULT TRUE,
    
    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_journal_tax_header ON journal_entry_tax(header_id);
CREATE INDEX idx_journal_tax_code ON journal_entry_tax(tax_code);

-- ============================================================================
-- 清账凭证表
-- ============================================================================
CREATE TABLE clearing_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 清账凭证号
    clearing_number VARCHAR(20) NOT NULL,
    company_code VARCHAR(4) NOT NULL,
    fiscal_year INT NOT NULL,
    
    -- 清账日期
    clearing_date DATE NOT NULL,
    
    -- 清账金额
    clearing_amount DECIMAL(15, 2) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    
    -- 清账类型
    clearing_type VARCHAR(20), -- PAYMENT, REVERSAL, MANUAL
    
    -- 审计字段
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT uk_clearing_document 
        UNIQUE (company_code, fiscal_year, clearing_number)
);

-- 清账明细表 (已清项目)
CREATE TABLE clearing_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 外键
    clearing_document_id UUID NOT NULL REFERENCES clearing_documents(id) ON DELETE CASCADE,
    journal_entry_line_id UUID NOT NULL REFERENCES journal_entry_lines(id),
    
    -- 清账金额
    cleared_amount DECIMAL(15, 2) NOT NULL,
    
    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_clearing_items_document ON clearing_items(clearing_document_id);
CREATE INDEX idx_clearing_items_line ON clearing_items(journal_entry_line_id);

-- ============================================================================
-- 一次性账户数据表 (参考: BSEC)
-- ============================================================================
CREATE TABLE journal_entry_one_time (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 外键
    line_id UUID NOT NULL REFERENCES journal_entry_lines(id) ON DELETE CASCADE,
    
    -- 名称地址
    name VARCHAR(100),
    name_2 VARCHAR(100),
    country VARCHAR(3),
    city VARCHAR(50),
    postal_code VARCHAR(20),
    street VARCHAR(100),
    region VARCHAR(10),
    
    -- 银行信息
    bank_country VARCHAR(3),
    bank_key VARCHAR(20),
    bank_account VARCHAR(30),
    iban VARCHAR(40),
    swift_bic VARCHAR(20),
    account_holder VARCHAR(100),
    
    -- 税号
    tax_number VARCHAR(30),
    
    -- 审计字段
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_one_time_line ON journal_entry_one_time(line_id);
