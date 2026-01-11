CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Subcontracting Orders
CREATE TABLE subcontracting_orders (
    order_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    purchase_order_number VARCHAR(20) UNIQUE NOT NULL,
    supplier VARCHAR(20) NOT NULL,
    company_code VARCHAR(4) NOT NULL,
    purchasing_org VARCHAR(4),
    purchasing_group VARCHAR(3),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Subcontracting Items (Finished Goods)
CREATE TABLE subcontracting_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID NOT NULL REFERENCES subcontracting_orders(order_id),
    item_number INT NOT NULL,
    finished_good_material VARCHAR(40) NOT NULL,
    order_quantity DECIMAL(15,3),
    received_quantity DECIMAL(15,3) DEFAULT 0,
    unit VARCHAR(3) DEFAULT 'EA',
    plant VARCHAR(4)
);

-- Subcontracting Components
CREATE TABLE subcontracting_components (
    component_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    item_id UUID NOT NULL REFERENCES subcontracting_items(item_id),
    component_material VARCHAR(40) NOT NULL,
    required_quantity DECIMAL(15,3),
    issued_quantity DECIMAL(15,3) DEFAULT 0,
    unit VARCHAR(3) DEFAULT 'EA'
);

CREATE INDEX idx_subcon_supplier ON subcontracting_orders(supplier);
CREATE INDEX idx_subcon_items ON subcontracting_items(order_id);
CREATE INDEX idx_subcon_components ON subcontracting_components(item_id);
