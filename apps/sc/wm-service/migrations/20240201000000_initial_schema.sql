CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Transfer Orders (LTAK)
CREATE TABLE transfer_orders (
    to_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    to_number VARCHAR(20) UNIQUE NOT NULL,
    warehouse_number VARCHAR(4) NOT NULL,
    movement_type VARCHAR(10) NOT NULL,
    reference_doc_type VARCHAR(10),
    reference_doc_number VARCHAR(20),
    status VARCHAR(20) DEFAULT 'CREATED',
    created_by VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Transfer Order Items (LTAP)
CREATE TABLE transfer_order_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    to_id UUID NOT NULL REFERENCES transfer_orders(to_id),
    item_number INT NOT NULL,
    material VARCHAR(40) NOT NULL,
    target_quantity DECIMAL(15,3) NOT NULL,
    actual_quantity DECIMAL(15,3) DEFAULT 0,
    unit VARCHAR(3) DEFAULT 'EA',
    src_storage_type VARCHAR(4),
    src_storage_bin VARCHAR(10),
    dst_storage_type VARCHAR(4),
    dst_storage_bin VARCHAR(10),
    batch VARCHAR(20),
    confirmed BOOLEAN DEFAULT FALSE
);

CREATE INDEX idx_to_warehouse ON transfer_orders(warehouse_number);
CREATE INDEX idx_to_items ON transfer_order_items(to_id);
