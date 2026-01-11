CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Contracts
CREATE TABLE contracts (
    contract_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contract_number VARCHAR(20) UNIQUE NOT NULL,
    company_code VARCHAR(4) NOT NULL,
    supplier VARCHAR(20) NOT NULL,
    purchasing_org VARCHAR(4) NOT NULL,
    purchasing_group VARCHAR(3),
    validity_start DATE,
    validity_end DATE,
    target_value DECIMAL(15,2),
    currency VARCHAR(3) DEFAULT 'CNY',
    release_status VARCHAR(20) DEFAULT 'NOT_RELEASED',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Contract Items
CREATE TABLE contract_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    contract_id UUID NOT NULL REFERENCES contracts(contract_id),
    item_number INT NOT NULL,
    material VARCHAR(40),
    short_text VARCHAR(255),
    target_quantity DECIMAL(15,3),
    unit VARCHAR(3) DEFAULT 'EA',
    net_price DECIMAL(15,2),
    price_currency VARCHAR(3) DEFAULT 'CNY',
    plant VARCHAR(4)
);

CREATE INDEX idx_contract_supplier ON contracts(supplier);
CREATE INDEX idx_contract_items ON contract_items(contract_id);
