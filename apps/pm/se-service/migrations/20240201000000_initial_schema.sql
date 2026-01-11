CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- RFQs (Request for Quotation)
CREATE TABLE rfqs (
    rfq_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rfq_number VARCHAR(20) UNIQUE NOT NULL,
    company_code VARCHAR(4) NOT NULL,
    purchasing_org VARCHAR(4),
    quote_deadline DATE,
    status VARCHAR(20) DEFAULT 'DRAFT',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- RFQ Items
CREATE TABLE rfq_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    rfq_id UUID NOT NULL REFERENCES rfqs(rfq_id),
    item_number INT NOT NULL,
    material VARCHAR(40) NOT NULL,
    description VARCHAR(255),
    quantity DECIMAL(15,3),
    unit VARCHAR(3) DEFAULT 'EA',
    delivery_date DATE
);

-- Supplier Quotes
CREATE TABLE supplier_quotes (
    quote_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    quote_number VARCHAR(20) UNIQUE NOT NULL,
    rfq_id UUID NOT NULL REFERENCES rfqs(rfq_id),
    supplier_id VARCHAR(20) NOT NULL,
    validity_end_date DATE,
    status VARCHAR(20) DEFAULT 'SUBMITTED',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Quote Items
CREATE TABLE quote_items (
    quote_item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    quote_id UUID NOT NULL REFERENCES supplier_quotes(quote_id),
    rfq_item_number INT NOT NULL,
    quantity DECIMAL(15,3),
    unit VARCHAR(3) DEFAULT 'EA',
    net_price DECIMAL(15,2),
    currency VARCHAR(3) DEFAULT 'CNY',
    notes TEXT
);

CREATE INDEX idx_rfq_company ON rfqs(company_code);
CREATE INDEX idx_rfq_items ON rfq_items(rfq_id);
CREATE INDEX idx_quotes ON supplier_quotes(rfq_id);
CREATE INDEX idx_quote_items ON quote_items(quote_id);
