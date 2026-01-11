-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Material Stock (Current Snapshot - MARD equivalent)
CREATE TABLE material_stock (
    stock_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    
    plant VARCHAR(4) NOT NULL,
    storage_location VARCHAR(4) NOT NULL,
    material VARCHAR(40) NOT NULL,
    batch VARCHAR(10) NOT NULL DEFAULT '', -- Empty string if no batch
    
    -- Quantities
    unrestricted_quantity DECIMAL(19, 3) NOT NULL DEFAULT 0,
    quality_inspection_quantity DECIMAL(19, 3) NOT NULL DEFAULT 0,
    blocked_quantity DECIMAL(19, 3) NOT NULL DEFAULT 0,
    
    base_unit VARCHAR(3) NOT NULL,
    
    last_movement_date TIMESTAMPTZ,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(plant, storage_location, material, batch)
);

CREATE INDEX idx_stock_material ON material_stock(material);
CREATE INDEX idx_stock_plant_loc ON material_stock(plant, storage_location);

-- Material Document Header (MKPF)
CREATE TABLE material_documents (
    document_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_number VARCHAR(20) UNIQUE NOT NULL, -- Generated ID
    fiscal_year INT NOT NULL,
    
    document_date DATE NOT NULL,
    posting_date DATE NOT NULL,
    
    document_type VARCHAR(2), -- WA, WE, etc.
    reference_document VARCHAR(20), -- Delivery, PO, etc.
    header_text VARCHAR(255),
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Material Document Items (MSEG)
CREATE TABLE material_document_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    document_id UUID NOT NULL REFERENCES material_documents(document_id),
    
    line_item_number INT NOT NULL, -- 1, 2...
    
    movement_type VARCHAR(3) NOT NULL, -- 101, 201, 561...
    debit_credit_indicator VARCHAR(1) NOT NULL, -- S (Debit/Receive), H (Credit/Issue)
    
    material VARCHAR(40) NOT NULL,
    plant VARCHAR(4) NOT NULL,
    storage_location VARCHAR(4) NOT NULL,
    batch VARCHAR(10) DEFAULT '',
    
    quantity DECIMAL(19, 3) NOT NULL,
    unit_of_measure VARCHAR(3) NOT NULL,
    
    -- Related reference
    special_stock_indicator VARCHAR(1),
    cost_center VARCHAR(10),
    order_number VARCHAR(12),
    
    amount_lc DECIMAL(19, 2), -- Local currency amount (optional for IM, vital for FI)
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_matdoc_items_mat ON material_document_items(material);
