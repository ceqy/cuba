CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- BOM Headers
CREATE TABLE bom_headers (
    bom_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    material VARCHAR(40) NOT NULL,
    plant VARCHAR(4) NOT NULL,
    bom_usage VARCHAR(10) DEFAULT 'PRODUCTION',
    bom_status VARCHAR(20) DEFAULT 'ACTIVE',
    base_quantity DECIMAL(15,3) DEFAULT 1,
    alternative_bom VARCHAR(2) DEFAULT '1',
    valid_from DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(material, plant, bom_usage, alternative_bom)
);

-- BOM Items
CREATE TABLE bom_items (
    item_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    bom_id UUID NOT NULL REFERENCES bom_headers(bom_id),
    item_node VARCHAR(10) NOT NULL,
    item_category VARCHAR(10) DEFAULT 'L',
    component_material VARCHAR(40) NOT NULL,
    component_quantity DECIMAL(15,3) NOT NULL,
    component_unit VARCHAR(3) DEFAULT 'EA',
    item_text VARCHAR(255),
    recursive_allowed BOOLEAN DEFAULT FALSE
);

CREATE INDEX idx_bom_material ON bom_headers(material, plant);
CREATE INDEX idx_bom_items ON bom_items(bom_id);
