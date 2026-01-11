CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Batches (MCH1)
CREATE TABLE batches (
    batch_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_number VARCHAR(20) UNIQUE NOT NULL,
    material VARCHAR(40) NOT NULL,
    plant VARCHAR(4) NOT NULL,
    production_date DATE,
    expiration_date DATE,
    supplier_batch VARCHAR(20),
    origin_batch VARCHAR(20),
    status VARCHAR(20) DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Batch History Events
CREATE TABLE batch_history (
    event_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    batch_id UUID NOT NULL REFERENCES batches(batch_id),
    event_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_type VARCHAR(30) NOT NULL,
    user_id VARCHAR(20),
    details TEXT,
    document_number VARCHAR(20),
    document_type VARCHAR(10)
);

CREATE INDEX idx_batch_material ON batches(material, plant);
CREATE INDEX idx_batch_history ON batch_history(batch_id);
