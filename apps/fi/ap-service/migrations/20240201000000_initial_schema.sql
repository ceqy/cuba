-- AP Service Database Schema (cuba_fi_ap)
-- Version 1.0 - Core tables for Accounts Payable

-- =============================================================================
-- Suppliers (供应商主数据)
-- =============================================================================
CREATE TABLE IF NOT EXISTS suppliers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    supplier_id VARCHAR(20) NOT NULL UNIQUE,
    business_partner_id VARCHAR(20),
    name VARCHAR(255) NOT NULL,
    account_group VARCHAR(10) NOT NULL,
    
    -- Address
    street VARCHAR(255),
    city VARCHAR(100),
    postal_code VARCHAR(20),
    country VARCHAR(3),
    
    -- Contact
    telephone VARCHAR(50),
    email VARCHAR(255),
    
    -- Company Code Data
    company_code VARCHAR(10) NOT NULL,
    reconciliation_account VARCHAR(20) NOT NULL,
    payment_terms VARCHAR(10),
    check_double_invoice BOOLEAN DEFAULT TRUE,
    
    -- Purchasing
    purchasing_organization VARCHAR(10),
    order_currency VARCHAR(3) DEFAULT 'CNY',
    
    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(50),
    updated_by VARCHAR(50)
);

CREATE INDEX idx_suppliers_company_code ON suppliers(company_code);
CREATE INDEX idx_suppliers_bp ON suppliers(business_partner_id);

-- =============================================================================
-- Invoices (发票抬头)
-- =============================================================================
CREATE TABLE IF NOT EXISTS invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_number VARCHAR(20) NOT NULL UNIQUE,
    company_code VARCHAR(10) NOT NULL,
    fiscal_year INT NOT NULL,
    document_type VARCHAR(4) NOT NULL DEFAULT 'KR', -- KR=Vendor Invoice
    
    -- Header
    supplier_id UUID NOT NULL REFERENCES suppliers(id),
    document_date DATE NOT NULL,
    posting_date DATE NOT NULL,
    due_date DATE NOT NULL,
    baseline_date DATE,
    
    -- Amounts
    currency VARCHAR(3) NOT NULL DEFAULT 'CNY',
    total_amount DECIMAL(18, 2) NOT NULL,
    tax_amount DECIMAL(18, 2) DEFAULT 0,
    
    -- Reference
    reference_document VARCHAR(50),
    header_text VARCHAR(255),
    
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'OPEN', -- OPEN, CLEARED, REVERSED
    clearing_document VARCHAR(20),
    clearing_date DATE,
    
    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(50),
    updated_by VARCHAR(50)
);

CREATE INDEX idx_invoices_supplier ON invoices(supplier_id);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_due_date ON invoices(due_date);
CREATE INDEX idx_invoices_company_fiscal ON invoices(company_code, fiscal_year);

-- =============================================================================
-- Invoice Line Items (发票行项目)
-- =============================================================================
CREATE TABLE IF NOT EXISTS invoice_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    line_item_number INT NOT NULL,
    
    -- Account
    gl_account VARCHAR(20) NOT NULL,
    debit_credit_indicator CHAR(1) NOT NULL, -- S=Debit, H=Credit
    
    -- Amount
    amount DECIMAL(18, 2) NOT NULL,
    
    -- Assignment
    cost_center VARCHAR(20),
    profit_center VARCHAR(20),
    item_text VARCHAR(255),
    
    -- Three-way matching (MIRO)
    purchase_order VARCHAR(20),
    po_item_number INT,
    goods_receipt VARCHAR(20),
    gr_item_number INT,
    quantity DECIMAL(18, 3),
    unit_of_measure VARCHAR(5),
    
    UNIQUE(invoice_id, line_item_number)
);

-- =============================================================================
-- Open Items (未清项)
-- =============================================================================
CREATE TABLE IF NOT EXISTS open_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Document Reference
    document_number VARCHAR(20) NOT NULL,
    company_code VARCHAR(10) NOT NULL,
    fiscal_year INT NOT NULL,
    line_item_number INT NOT NULL,
    
    -- Partner
    supplier_id UUID REFERENCES suppliers(id),
    account_type CHAR(1) NOT NULL DEFAULT 'K', -- K=Vendor, D=Customer
    
    -- Dates
    posting_date DATE NOT NULL,
    due_date DATE NOT NULL,
    baseline_date DATE,
    
    -- Amounts
    currency VARCHAR(3) NOT NULL,
    original_amount DECIMAL(18, 2) NOT NULL,
    open_amount DECIMAL(18, 2) NOT NULL,
    
    -- Status
    is_cleared BOOLEAN DEFAULT FALSE,
    clearing_document VARCHAR(20),
    clearing_date DATE,
    
    -- Reference
    reference_document VARCHAR(50),
    item_text VARCHAR(255),
    payment_block VARCHAR(1),
    
    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(document_number, company_code, fiscal_year, line_item_number)
);

CREATE INDEX idx_open_items_supplier ON open_items(supplier_id);
CREATE INDEX idx_open_items_cleared ON open_items(is_cleared);
CREATE INDEX idx_open_items_due ON open_items(due_date);

-- =============================================================================
-- Payment Documents (付款凭证)
-- =============================================================================
CREATE TABLE IF NOT EXISTS payment_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_number VARCHAR(20) NOT NULL UNIQUE,
    company_code VARCHAR(10) NOT NULL,
    fiscal_year INT NOT NULL,
    
    -- Payment Details
    payment_date DATE NOT NULL,
    payment_method VARCHAR(5) NOT NULL, -- T=Transfer, C=Check, E=EDI
    house_bank VARCHAR(10),
    bank_account VARCHAR(20),
    
    -- Amount
    currency VARCHAR(3) NOT NULL,
    total_amount DECIMAL(18, 2) NOT NULL,
    
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'CREATED', -- CREATED, EXECUTED, REVERSED
    
    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(50)
);

-- =============================================================================
-- Cleared Items Junction (清账关联)
-- =============================================================================
CREATE TABLE IF NOT EXISTS cleared_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    clearing_document VARCHAR(20) NOT NULL,
    open_item_id UUID NOT NULL REFERENCES open_items(id),
    cleared_amount DECIMAL(18, 2) NOT NULL,
    cleared_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(clearing_document, open_item_id)
);

CREATE INDEX idx_cleared_items_doc ON cleared_items(clearing_document);
