CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Allocation Runs
CREATE TABLE allocation_runs (
    run_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    controlling_area VARCHAR(4) NOT NULL,
    fiscal_year INT NOT NULL,
    fiscal_period INT NOT NULL,
    allocation_cycle VARCHAR(10) NOT NULL,
    allocation_type VARCHAR(20) NOT NULL, -- COST_CENTER, ACTIVITY
    test_run BOOLEAN DEFAULT FALSE,
    status VARCHAR(20) DEFAULT 'COMPLETED',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Allocation Senders
CREATE TABLE allocation_senders (
    sender_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    run_id UUID NOT NULL REFERENCES allocation_runs(run_id),
    sender_object VARCHAR(20) NOT NULL,
    sent_amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY'
);

-- Allocation Receivers
CREATE TABLE allocation_receivers (
    receiver_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    run_id UUID NOT NULL REFERENCES allocation_runs(run_id),
    receiver_object VARCHAR(20) NOT NULL,
    received_amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) DEFAULT 'CNY'
);

CREATE INDEX idx_alloc_run ON allocation_runs(controlling_area, fiscal_year, fiscal_period);
