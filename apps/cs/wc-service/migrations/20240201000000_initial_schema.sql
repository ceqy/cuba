CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Claims
CREATE TABLE claims (
    claim_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    customer_id VARCHAR(20) NOT NULL,
    product_id VARCHAR(40) NOT NULL,
    failure_date DATE NOT NULL,
    failure_description TEXT,
    claimed_amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY',
    status VARCHAR(20) DEFAULT 'SUBMITTED',
    attachment_urls TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Adjudications
CREATE TABLE adjudications (
    adjudication_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    claim_id UUID NOT NULL REFERENCES claims(claim_id),
    adjudicated_by VARCHAR(20) NOT NULL,
    adjudication_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_amount DECIMAL(15,2),
    currency VARCHAR(3) DEFAULT 'CNY',
    notes TEXT
);

CREATE INDEX idx_claim_customer ON claims(customer_id);
CREATE INDEX idx_claim_status ON claims(status);
