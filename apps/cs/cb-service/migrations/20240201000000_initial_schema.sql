CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Service Contracts
CREATE TABLE service_contracts (
    contract_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contract_number VARCHAR(20) UNIQUE NOT NULL,
    customer_id VARCHAR(20) NOT NULL,
    validity_start DATE NOT NULL,
    validity_end DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Billing Plan Items
CREATE TABLE billing_plan_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contract_id UUID NOT NULL REFERENCES service_contracts(contract_id),
    planned_date DATE NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY',
    status VARCHAR(20) DEFAULT 'OPEN', -- OPEN, BILLED, CANCELLED
    invoice_number VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_bp_contract ON billing_plan_items(contract_id);
