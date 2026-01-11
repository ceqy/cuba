-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Maintenance Notifications (QMEL)
CREATE TABLE maintenance_notifications (
    notification_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    notification_number VARCHAR(20) UNIQUE NOT NULL,
    notification_type VARCHAR(4) NOT NULL, -- M1, M2
    description TEXT,
    equipment_number VARCHAR(20),
    functional_location VARCHAR(30),
    reported_by VARCHAR(20),
    reported_date TIMESTAMPTZ DEFAULT NOW(),
    priority VARCHAR(2) DEFAULT '3',
    status VARCHAR(20) DEFAULT 'OSNO', -- Outstanding Notification
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Maintenance Orders (AUFK/AFKO)
CREATE TABLE maintenance_orders (
    order_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_number VARCHAR(20) UNIQUE NOT NULL,
    order_type VARCHAR(4) NOT NULL, -- PM01, PM02
    description TEXT,
    
    notification_number VARCHAR(20),
    equipment_number VARCHAR(20),
    functional_location VARCHAR(30),
    
    maintenance_plant VARCHAR(4) NOT NULL,
    planning_plant VARCHAR(4),
    main_work_center VARCHAR(10),
    
    system_status VARCHAR(20) DEFAULT 'CRTD',
    priority VARCHAR(2) DEFAULT '3',
    
    basic_start_date DATE,
    basic_finish_date DATE,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Maintenance Operations (AFVC)
CREATE TABLE maintenance_operations (
    operation_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    order_id UUID NOT NULL REFERENCES maintenance_orders(order_id),
    operation_number VARCHAR(4) NOT NULL,
    description TEXT,
    work_center VARCHAR(10),
    planned_work_duration DECIMAL(10,2) DEFAULT 0,
    actual_work_duration DECIMAL(10,2) DEFAULT 0,
    work_unit VARCHAR(3) DEFAULT 'H',
    status VARCHAR(20) DEFAULT 'CRTD',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(order_id, operation_number)
);

CREATE INDEX idx_notif_equipment ON maintenance_notifications(equipment_number);
CREATE INDEX idx_order_equipment ON maintenance_orders(equipment_number);
