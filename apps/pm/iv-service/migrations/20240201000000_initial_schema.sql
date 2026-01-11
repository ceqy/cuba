CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Invoices (RBKP)
CREATE TABLE invoices (
    invoice_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_code VARCHAR(4) NOT NULL,
    supplier_invoice_number VARCHAR(20) NOT NULL,
    document_date DATE NOT NULL,
    posting_date DATE,
    gross_amount DECIMAL(15,2) NOT NULL,
    tax_amount DECIMAL(15,2) DEFAULT 0,
    currency VARCHAR(3) DEFAULT 'CNY',
    payment_terms VARCHAR(10),
    header_text VARCHAR(255),
    status VARCHAR(20) DEFAULT 'RECEIVED',
    document_number VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Invoice Items (RSEG)
CREATE TABLE invoice_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    invoice_id UUID NOT NULL REFERENCES invoices(invoice_id),
    item_number INT NOT NULL,
    po_number VARCHAR(20),
    po_item INT,
    material VARCHAR(40),
    short_text VARCHAR(255),
    quantity DECIMAL(15,3) NOT NULL,
    unit VARCHAR(3) DEFAULT 'EA',
    amount DECIMAL(15,2) NOT NULL,
    tax_code VARCHAR(5),
    gr_document VARCHAR(20),
    gr_year INT,
    gr_item INT
);

CREATE INDEX idx_inv_vendor ON invoices(supplier_invoice_number);
CREATE INDEX idx_inv_items ON invoice_items(invoice_id);
