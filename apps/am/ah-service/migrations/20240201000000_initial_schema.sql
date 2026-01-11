CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Sensor Data
CREATE TABLE sensor_data (
    data_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    equipment_number VARCHAR(20) NOT NULL,
    sensor_id VARCHAR(50) NOT NULL,
    value VARCHAR(100),
    unit VARCHAR(10),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Asset Health Status
CREATE TABLE asset_health (
    health_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    equipment_number VARCHAR(20) UNIQUE NOT NULL,
    health_score INT DEFAULT 100,
    status_description VARCHAR(20),
    remaining_useful_life VARCHAR(50),
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Predictive Alerts
CREATE TABLE predictive_alerts (
    alert_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    equipment_number VARCHAR(20) NOT NULL,
    failure_mode VARCHAR(50),
    recommended_action TEXT,
    confidence_score DECIMAL(5,2),
    alert_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sensor_equipment ON sensor_data(equipment_number);
CREATE INDEX idx_health_equipment ON asset_health(equipment_number);
CREATE INDEX idx_alerts_equipment ON predictive_alerts(equipment_number);
