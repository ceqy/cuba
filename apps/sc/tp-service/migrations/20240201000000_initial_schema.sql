CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Shipments
CREATE TABLE shipments (
    shipment_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_number VARCHAR(20) UNIQUE NOT NULL,
    shipment_type VARCHAR(20),
    transportation_planning_point VARCHAR(10),
    carrier VARCHAR(20),
    overall_status VARCHAR(20) DEFAULT 'PLANNED',
    planned_departure TIMESTAMPTZ,
    planned_arrival TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Shipment Items
CREATE TABLE shipment_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_id UUID NOT NULL REFERENCES shipments(shipment_id),
    item_number INT NOT NULL,
    delivery_number VARCHAR(20) NOT NULL,
    total_weight DECIMAL(15,3),
    weight_unit VARCHAR(3) DEFAULT 'KG',
    volume DECIMAL(15,3),
    volume_unit VARCHAR(3) DEFAULT 'M3'
);

CREATE INDEX idx_shipment_carrier ON shipments(carrier);
CREATE INDEX idx_shipment_status ON shipments(overall_status);
CREATE INDEX idx_shipment_items ON shipment_items(shipment_id);
