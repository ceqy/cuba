CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Control Cycles (PKHD)
CREATE TABLE control_cycles (
    cycle_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    cycle_number VARCHAR(20) UNIQUE NOT NULL,
    material VARCHAR(40) NOT NULL,
    plant VARCHAR(4) NOT NULL,
    supply_area VARCHAR(10),
    number_of_kanbans INT DEFAULT 3,
    qty_per_kanban DECIMAL(15,3) DEFAULT 100,
    unit VARCHAR(3) DEFAULT 'EA',
    replenishment_strategy VARCHAR(20) DEFAULT 'PRODUCTION',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Kanban Containers (PKPS)
CREATE TABLE kanban_containers (
    container_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    container_code VARCHAR(30) UNIQUE NOT NULL,
    cycle_id UUID NOT NULL REFERENCES control_cycles(cycle_id),
    status VARCHAR(20) DEFAULT 'FULL',
    last_status_change TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_cycle_material ON control_cycles(material, plant);
CREATE INDEX idx_container_cycle ON kanban_containers(cycle_id);
