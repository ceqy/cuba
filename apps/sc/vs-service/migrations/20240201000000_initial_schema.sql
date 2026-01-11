CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE vendor_evaluations (eval_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(), vendor_id VARCHAR(20) NOT NULL, evaluation_date DATE, overall_score DECIMAL(5,2), quality_score DECIMAL(5,2), delivery_score DECIMAL(5,2), price_score DECIMAL(5,2), status VARCHAR(20) DEFAULT 'ACTIVE', created_at TIMESTAMPTZ NOT NULL DEFAULT NOW());
CREATE INDEX idx_vendor_eval ON vendor_evaluations(vendor_id);
