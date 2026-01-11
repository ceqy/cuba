-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Purchase Order Headers (EKKO)
CREATE TABLE purchase_orders (
    order_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_number VARCHAR(20) UNIQUE NOT NULL, -- e.g., 4500000001
    
    document_type INTEGER NOT NULL, -- Enum integer
    company_code VARCHAR(4) NOT NULL,
    purchasing_org VARCHAR(4) NOT NULL,
    purchasing_group VARCHAR(3) NOT NULL,
    
    supplier VARCHAR(20) NOT NULL,
    
    order_date DATE NOT NULL,
    
    currency VARCHAR(3) NOT NULL,
    payment_terms VARCHAR(4),
    incoterms VARCHAR(3),
    incoterms_location VARCHAR(70),
    
    complete_delivery BOOLEAN DEFAULT FALSE,
    release_status INTEGER NOT NULL DEFAULT 1, -- 1=Not Released
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_po_supplier ON purchase_orders(supplier);
CREATE INDEX idx_po_date ON purchase_orders(order_date);

-- Purchase Order Items (EKPO)
CREATE TABLE purchase_order_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID NOT NULL REFERENCES purchase_orders(order_id),
    
    item_number INT NOT NULL, -- 10, 20...
    item_category INTEGER NOT NULL DEFAULT 1, -- Standard
    
    material VARCHAR(40) NOT NULL,
    short_text VARCHAR(255),
    
    plant VARCHAR(4) NOT NULL,
    storage_location VARCHAR(4),
    material_group VARCHAR(9),
    
    quantity DECIMAL(15, 3) NOT NULL,
    quantity_unit VARCHAR(3) NOT NULL,
    
    net_price DECIMAL(15, 2) NOT NULL,
    price_unit INT DEFAULT 1,
    currency VARCHAR(3) NOT NULL,
    
    gr_based_iv BOOLEAN DEFAULT TRUE,
    tax_code VARCHAR(2),
    account_assignment_category VARCHAR(1),
    
    requisition_number VARCHAR(20),
    requisition_item INT,
    
    deletion_indicator BOOLEAN DEFAULT FALSE,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(order_id, item_number)
);

-- Purchase Order Schedule Lines (EKET)
CREATE TABLE purchase_order_schedule_lines (
    schedule_line_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    item_id UUID NOT NULL REFERENCES purchase_order_items(item_id),
    
    schedule_line_number INT NOT NULL, -- 1, 2...
    delivery_date DATE NOT NULL,
    
    scheduled_quantity DECIMAL(15, 3) NOT NULL,
    goods_receipt_quantity DECIMAL(15, 3) NOT NULL DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
