CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Spend Facts (Aggregated)
CREATE TABLE spend_facts (
    fact_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    spend_date DATE NOT NULL,
    company_code VARCHAR(4),
    purchasing_org VARCHAR(4),
    plant VARCHAR(4),
    category VARCHAR(50),
    supplier VARCHAR(20),
    spend_amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY',
    document_count INT DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_spend_date ON spend_facts(spend_date);
CREATE INDEX idx_spend_category ON spend_facts(category);
CREATE INDEX idx_spend_supplier ON spend_facts(supplier);
