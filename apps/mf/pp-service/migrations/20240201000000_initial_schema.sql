-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Planned Orders (PLAF)
CREATE TABLE planned_orders (
    planned_order_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    planned_order_number VARCHAR(20) UNIQUE NOT NULL, -- e.g., 0010000001
    
    material VARCHAR(40) NOT NULL,
    plant VARCHAR(4) NOT NULL,
    planning_plant VARCHAR(4) NOT NULL,
    
    order_quantity DECIMAL(15, 3) NOT NULL,
    quantity_unit VARCHAR(3) NOT NULL,
    
    order_start_date DATE NOT NULL,
    order_finish_date DATE NOT NULL,
    
    mrp_controller VARCHAR(3),
    conversion_indicator VARCHAR(1) DEFAULT '',
    
    status VARCHAR(20) DEFAULT 'CREATED',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_plaf_material ON planned_orders(material);
CREATE INDEX idx_plaf_plant ON planned_orders(plant);
CREATE INDEX idx_plaf_dates ON planned_orders(order_finish_date);
