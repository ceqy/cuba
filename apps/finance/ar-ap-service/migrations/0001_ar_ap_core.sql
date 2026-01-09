-- ============================================================================
-- AR/AP Service: Core Tables
-- 描述: 应收应付核心表结构
-- ============================================================================

-- 业务伙伴类型
CREATE TYPE partner_type AS ENUM ('PERSON', 'ORGANIZATION');
CREATE TYPE account_type AS ENUM ('CUSTOMER', 'SUPPLIER');

-- 业务伙伴主数据
CREATE TABLE business_partners (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    partner_id VARCHAR(10) NOT NULL UNIQUE,
    partner_type partner_type NOT NULL,
    name_org1 VARCHAR(40),
    name_last VARCHAR(40),
    name_first VARCHAR(40),
    search_term VARCHAR(20),
    country VARCHAR(3),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 客户
CREATE TABLE customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id VARCHAR(10) NOT NULL UNIQUE,
    partner_id VARCHAR(10) REFERENCES business_partners(partner_id),
    company_code VARCHAR(4) NOT NULL,
    reconciliation_account VARCHAR(10),
    payment_terms VARCHAR(4),
    credit_limit DECIMAL(15,2),
    credit_currency VARCHAR(3) DEFAULT 'CNY',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 供应商
CREATE TABLE suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id VARCHAR(10) NOT NULL UNIQUE,
    partner_id VARCHAR(10) REFERENCES business_partners(partner_id),
    company_code VARCHAR(4) NOT NULL,
    reconciliation_account VARCHAR(10),
    payment_terms VARCHAR(4),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 未清项
CREATE TABLE open_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_code VARCHAR(4) NOT NULL,
    document_number VARCHAR(10) NOT NULL,
    fiscal_year INT NOT NULL,
    line_item INT NOT NULL,
    account_type account_type NOT NULL,
    partner_id VARCHAR(10) NOT NULL,
    posting_date DATE NOT NULL,
    due_date DATE,
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL,
    open_amount DECIMAL(15,2) NOT NULL,
    clearing_date DATE,
    clearing_doc VARCHAR(10),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(company_code, document_number, fiscal_year, line_item)
);

-- 索引
CREATE INDEX idx_business_partners_search ON business_partners(search_term);
CREATE INDEX idx_customers_company ON customers(company_code);
CREATE INDEX idx_suppliers_company ON suppliers(company_code);
CREATE INDEX idx_open_items_partner ON open_items(partner_id);
CREATE INDEX idx_open_items_due_date ON open_items(due_date);
CREATE INDEX idx_open_items_clearing ON open_items(clearing_date) WHERE clearing_date IS NULL;
