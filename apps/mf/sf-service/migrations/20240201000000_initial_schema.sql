-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Production Orders (AUFK/AFKO)
CREATE TABLE production_orders (
    order_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_number VARCHAR(20) UNIQUE NOT NULL, -- e.g., 1000001
    
    order_type VARCHAR(4) NOT NULL, -- e.g., PP01
    material VARCHAR(40) NOT NULL,
    plant VARCHAR(4) NOT NULL,
    
    total_quantity DECIMAL(15, 3) NOT NULL,
    delivered_quantity DECIMAL(15, 3) DEFAULT 0,
    quantity_unit VARCHAR(3) NOT NULL,
    
    basic_start_date DATE NOT NULL,
    basic_finish_date DATE NOT NULL,
    
    status VARCHAR(20) DEFAULT 'CREATED', -- CRTD, REL, CNF, DLV
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Production Operations (AFVC)
CREATE TABLE production_operations (
    operation_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID NOT NULL REFERENCES production_orders(order_id),
    operation_number VARCHAR(4) NOT NULL, -- 0010, 0020
    
    work_center VARCHAR(10) NOT NULL,
    description TEXT,
    
    confirmed_yield DECIMAL(15, 3) DEFAULT 0,
    status VARCHAR(20) DEFAULT 'CREATED',
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(order_id, operation_number)
);

-- Production Confirmations (AFRU)
CREATE TABLE production_confirmations (
    confirmation_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    confirmation_number VARCHAR(20) UNIQUE NOT NULL,
    
    order_id UUID NOT NULL REFERENCES production_orders(order_id),
    operation_number VARCHAR(4) NOT NULL,
    
    yield_quantity DECIMAL(15, 3) NOT NULL,
    scrap_quantity DECIMAL(15, 3) DEFAULT 0,
    
    final_confirmation BOOLEAN DEFAULT FALSE,
    posting_date DATE NOT NULL,
    personnel_number VARCHAR(20),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_prod_order_material ON production_orders(material);
CREATE INDEX idx_prod_order_plant ON production_orders(plant);
