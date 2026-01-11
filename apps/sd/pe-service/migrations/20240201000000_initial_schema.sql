CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Pricing Conditions (KONP)
CREATE TABLE pricing_conditions (
    condition_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    condition_type VARCHAR(4) NOT NULL,
    material VARCHAR(40),
    customer VARCHAR(20),
    sales_org VARCHAR(4) NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY',
    valid_from DATE,
    valid_to DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cond_material ON pricing_conditions(material, sales_org);
CREATE INDEX idx_cond_customer ON pricing_conditions(customer, sales_org);
