CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Service Orders
CREATE TABLE service_orders (
    order_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_number VARCHAR(20) UNIQUE NOT NULL,
    order_type VARCHAR(10) DEFAULT 'REPAIR',
    customer_id VARCHAR(20) NOT NULL,
    description TEXT,
    planned_start TIMESTAMPTZ,
    assigned_technician_id VARCHAR(20),
    status VARCHAR(20) DEFAULT 'OPEN',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_so_customer ON service_orders(customer_id);
CREATE INDEX idx_so_technician ON service_orders(assigned_technician_id);
