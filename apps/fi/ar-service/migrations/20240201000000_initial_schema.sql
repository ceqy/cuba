-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Customers Table (Master Data)
CREATE TABLE customers (
    customer_id VARCHAR(20) PRIMARY KEY, -- e.g., CUST001
    business_partner_id VARCHAR(20), -- Link to BP
    name VARCHAR(255) NOT NULL,
    account_group VARCHAR(4) NOT NULL, -- e.g., KUNA (Customer)
    
    -- Address (Simplified)
    street VARCHAR(255),
    city VARCHAR(255),
    postal_code VARCHAR(20),
    country VARCHAR(2), -- ISO 2
    
    -- Control Data
    company_code VARCHAR(4) NOT NULL,
    reconciliation_account VARCHAR(10) NOT NULL, -- GL Account
    payment_terms VARCHAR(4),
    
    -- Sales Data
    sales_organization VARCHAR(4),
    distribution_channel VARCHAR(2),
    division VARCHAR(2),
    order_currency VARCHAR(3),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_customers_company_code ON customers(company_code);
CREATE INDEX idx_customers_business_partner_id ON customers(business_partner_id);

-- Invoices Table (Sales)
CREATE TABLE invoices (
    invoice_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_number VARCHAR(20) UNIQUE, -- Generated or External
    company_code VARCHAR(4) NOT NULL,
    fiscal_year INT NOT NULL,
    document_date DATE NOT NULL,
    posting_date DATE NOT NULL,
    
    customer_id VARCHAR(20) NOT NULL REFERENCES customers(customer_id),
    currency VARCHAR(3) NOT NULL,
    total_amount DECIMAL(19, 4) NOT NULL,
    
    reference VARCHAR(50), -- External Reference (e.g. PO Number)
    status VARCHAR(20) NOT NULL, -- DRAFT, POSTED, CANCELLED
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_invoices_customer ON invoices(customer_id);

-- Invoice Items
CREATE TABLE invoice_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    invoice_id UUID NOT NULL REFERENCES invoices(invoice_id),
    line_item_number INT NOT NULL,
    
    description VARCHAR(255),
    quantity DECIMAL(15, 3),
    unit_price DECIMAL(19, 4),
    total_price DECIMAL(19, 4) NOT NULL,
    
    gl_account VARCHAR(10) NOT NULL, -- Revenue Account
    tax_code VARCHAR(2),
    cost_center VARCHAR(10),
    profit_center VARCHAR(10)
);

CREATE INDEX idx_invoice_items_invoice ON invoice_items(invoice_id);

-- Open Items (Receivables)
CREATE TABLE open_items (
    open_item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    -- Document Reference
    document_number VARCHAR(20) NOT NULL,
    fiscal_year INT NOT NULL,
    company_code VARCHAR(4) NOT NULL,
    line_item_number INT NOT NULL,
    
    customer_id VARCHAR(20) NOT NULL REFERENCES customers(customer_id),
    
    doc_type VARCHAR(2) NOT NULL, -- DR (Inv), DZ (Pay), DG (Credit Memo)
    posting_date DATE NOT NULL,
    document_date DATE NOT NULL,
    due_date DATE NOT NULL,
    
    currency VARCHAR(3) NOT NULL,
    original_amount DECIMAL(19, 4) NOT NULL, -- Positive = Receivable
    open_amount DECIMAL(19, 4) NOT NULL,
    
    is_cleared BOOLEAN NOT NULL DEFAULT FALSE,
    clearing_document VARCHAR(20),
    clearing_date DATE,
    
    payment_block VARCHAR(1),
    reference_document VARCHAR(50),
    item_text VARCHAR(255),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE (company_code, fiscal_year, document_number, line_item_number)
);

CREATE INDEX idx_open_items_customer ON open_items(customer_id);
CREATE INDEX idx_open_items_due_date ON open_items(due_date) WHERE is_cleared = FALSE;

-- Payment Documents (Incoming Payments)
CREATE TABLE payment_documents (
    payment_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_number VARCHAR(20) UNIQUE,
    company_code VARCHAR(4) NOT NULL,
    fiscal_year INT NOT NULL,
    
    customer_id VARCHAR(20) REFERENCES customers(customer_id),
    posting_date DATE NOT NULL,
    
    currency VARCHAR(3) NOT NULL,
    amount DECIMAL(19, 4) NOT NULL,
    
    payment_method VARCHAR(1), -- C=Check, T=Transfer
    bank_account_id VARCHAR(20),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Cleared Items Link (Many-to-Many for partial payments)
CREATE TABLE cleared_items (
    clearing_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payment_id UUID REFERENCES payment_documents(payment_id),
    open_item_id UUID REFERENCES open_items(open_item_id),
    amount_cleared DECIMAL(19, 4) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
