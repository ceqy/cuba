CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Bank Statements
CREATE TABLE bank_statements (
    statement_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    company_code VARCHAR(4) NOT NULL,
    statement_format VARCHAR(20) DEFAULT 'MT940',
    status VARCHAR(20) DEFAULT 'PROCESSED',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Statement Transactions
CREATE TABLE statement_transactions (
    transaction_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    statement_id UUID NOT NULL REFERENCES bank_statements(statement_id),
    value_date DATE NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY',
    memo VARCHAR(255),
    partner_name VARCHAR(100)
);

-- Payment Runs
CREATE TABLE payment_runs (
    run_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    run_number VARCHAR(20) UNIQUE NOT NULL,
    company_codes TEXT,
    posting_date DATE,
    status VARCHAR(20) DEFAULT 'PENDING',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Payment Documents
CREATE TABLE payment_documents (
    doc_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    run_id UUID NOT NULL REFERENCES payment_runs(run_id),
    document_number VARCHAR(20) NOT NULL,
    fiscal_year INT,
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY',
    payee_name VARCHAR(100)
);

CREATE INDEX idx_stmt_company ON bank_statements(company_code);
CREATE INDEX idx_run_status ON payment_runs(status);
