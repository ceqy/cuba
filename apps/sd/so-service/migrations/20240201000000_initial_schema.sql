-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Sales Order Headers (VBAK)
CREATE TABLE sales_orders (
    order_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_number VARCHAR(20) UNIQUE NOT NULL, -- e.g., OR000001
    
    order_type VARCHAR(4) NOT NULL, -- OR, RE, etc.
    sales_org VARCHAR(4) NOT NULL,
    distribution_channel VARCHAR(2) NOT NULL,
    division VARCHAR(2) NOT NULL,
    
    sold_to_party VARCHAR(20) NOT NULL, -- Customer ID
    ship_to_party VARCHAR(20),
    
    customer_po VARCHAR(35),
    customer_po_date DATE,
    
    document_date DATE NOT NULL,
    requested_delivery_date DATE,
    
    currency VARCHAR(3) NOT NULL,
    net_value DECIMAL(19, 4) NOT NULL DEFAULT 0,
    
    pricing_procedure VARCHAR(10),
    shipping_conditions VARCHAR(2),
    
    overall_status VARCHAR(20) NOT NULL DEFAULT 'OPEN', -- OPEN, COMPLETED, CANCELLED
    delivery_block VARCHAR(2),
    billing_block VARCHAR(2),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_so_sold_to ON sales_orders(sold_to_party);
CREATE INDEX idx_so_document_date ON sales_orders(document_date);

-- Sales Order Items (VBAP)
CREATE TABLE sales_order_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID NOT NULL REFERENCES sales_orders(order_id),
    item_number INT NOT NULL, -- 10, 20, 30...
    
    material VARCHAR(40) NOT NULL,
    item_description VARCHAR(255),
    
    order_quantity DECIMAL(15, 3) NOT NULL,
    sales_unit VARCHAR(3) NOT NULL,
    
    plant VARCHAR(4),
    storage_location VARCHAR(4),
    
    net_value DECIMAL(19, 4) NOT NULL DEFAULT 0,
    tax_amount DECIMAL(19, 4) DEFAULT 0,
    
    item_category VARCHAR(4), -- TAN, etc.
    rejection_reason VARCHAR(2),
    
    higher_level_item INT, -- For BOMs
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(order_id, item_number)
);

CREATE INDEX idx_so_items_material ON sales_order_items(material);

-- Sales Order Schedule Lines (VBEP)
CREATE TABLE sales_order_schedule_lines (
    schedule_line_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    item_id UUID NOT NULL REFERENCES sales_order_items(item_id),
    
    schedule_line_number INT NOT NULL, -- 1, 2...
    delivery_date DATE NOT NULL,
    confirmed_quantity DECIMAL(15, 3) NOT NULL DEFAULT 0,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_so_sl_date ON sales_order_schedule_lines(delivery_date);
