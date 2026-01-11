CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Revenue Contracts
CREATE TABLE revenue_contracts (
    contract_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contract_number VARCHAR(20) UNIQUE NOT NULL,
    source_document_number VARCHAR(20) NOT NULL,
    source_document_type VARCHAR(10),
    company_code VARCHAR(4) NOT NULL,
    customer VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Performance Obligations (POB)
CREATE TABLE performance_obligations (
    pob_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contract_id UUID NOT NULL REFERENCES revenue_contracts(contract_id),
    pob_code VARCHAR(20) UNIQUE NOT NULL,
    description VARCHAR(255),
    allocated_price DECIMAL(15,2),
    currency VARCHAR(3) DEFAULT 'CNY',
    recognition_method VARCHAR(20) DEFAULT 'POINT_IN_TIME',
    recognized_revenue DECIMAL(15,2) DEFAULT 0,
    deferred_revenue DECIMAL(15,2) DEFAULT 0
);

-- Revenue Posting Documents
CREATE TABLE revenue_postings (
    posting_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id VARCHAR(20) UNIQUE NOT NULL,
    posting_date DATE NOT NULL,
    pob_id UUID NOT NULL REFERENCES performance_obligations(pob_id),
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY',
    posting_type VARCHAR(20),
    accounting_document_number VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_contract_customer ON revenue_contracts(customer);
CREATE INDEX idx_pob_contract ON performance_obligations(contract_id);
CREATE INDEX idx_posting_pob ON revenue_postings(pob_id);
