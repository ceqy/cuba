-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Inspection Lots (QALS)
CREATE TABLE inspection_lots (
    lot_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    inspection_lot_number VARCHAR(20) UNIQUE NOT NULL, -- e.g., 80000001
    
    material VARCHAR(40) NOT NULL,
    plant VARCHAR(4) NOT NULL,
    
    lot_quantity DECIMAL(15, 3) NOT NULL,
    quantity_unit VARCHAR(3) NOT NULL,
    
    origin VARCHAR(4) NOT NULL, -- 01=GR, 04=Prod
    creation_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Usage Decision
    ud_code VARCHAR(4),
    ud_date TIMESTAMPTZ,
    ud_note TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Inspection Characteristics (Item level)
CREATE TABLE inspection_characteristics (
    char_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    lot_id UUID NOT NULL REFERENCES inspection_lots(lot_id),
    characteristic_number VARCHAR(4) NOT NULL, -- 0010
    
    description VARCHAR(255),
    inspection_method VARCHAR(255),
    
    -- Results
    result_value VARCHAR(255),
    result_status VARCHAR(1) DEFAULT '0', -- 0=Created, 2=Recorded, 5=Valuated
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(lot_id, characteristic_number)
);

CREATE INDEX idx_lot_material ON inspection_lots(material);
CREATE INDEX idx_lot_plant ON inspection_lots(plant);
