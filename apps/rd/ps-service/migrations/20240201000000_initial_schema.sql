CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Project Budgets
CREATE TABLE project_budgets (
    budget_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wbs_element VARCHAR(24) NOT NULL,
    fiscal_year INT NOT NULL,
    budget_amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY',
    version VARCHAR(10) DEFAULT 'ORIGINAL',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(wbs_element, fiscal_year, version)
);

-- Cost Postings
CREATE TABLE cost_postings (
    posting_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wbs_element VARCHAR(24) NOT NULL,
    cost_element VARCHAR(10) NOT NULL,
    cost_element_type VARCHAR(20) DEFAULT 'PRIMARY',
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY',
    posting_date DATE NOT NULL,
    description TEXT,
    document_number VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_budget_wbs ON project_budgets(wbs_element);
CREATE INDEX idx_posting_wbs ON cost_postings(wbs_element);
