CREATE TABLE IF NOT EXISTS events (
    sequence_id BIGSERIAL PRIMARY KEY,
    aggregate_type VARCHAR(255) NOT NULL,
    aggregate_id UUID NOT NULL,
    version BIGINT NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (aggregate_id, version)
);
