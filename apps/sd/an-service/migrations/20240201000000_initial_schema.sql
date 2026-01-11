CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Sales Facts (Aggregated)
CREATE TABLE sales_facts (
    fact_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    sales_date DATE NOT NULL,
    sales_org VARCHAR(4),
    distribution_channel VARCHAR(2),
    customer VARCHAR(20),
    product VARCHAR(40),
    revenue DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY',
    quantity_sold DECIMAL(15,3),
    unit VARCHAR(3),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sales_date ON sales_facts(sales_date);
CREATE INDEX idx_sales_customer ON sales_facts(customer);
CREATE INDEX idx_sales_product ON sales_facts(product);
