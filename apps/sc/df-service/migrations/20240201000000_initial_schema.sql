CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Forecast Plans
CREATE TABLE forecast_plans (
    plan_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plan_code VARCHAR(50) UNIQUE NOT NULL,
    material VARCHAR(40) NOT NULL,
    plant VARCHAR(4) NOT NULL,
    forecast_version VARCHAR(20),
    model_used VARCHAR(20),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Forecast Periods
CREATE TABLE forecast_periods (
    period_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    plan_id UUID NOT NULL REFERENCES forecast_plans(plan_id),
    start_date DATE NOT NULL,
    end_date DATE NOT NULL,
    forecasted_quantity DECIMAL(15,3),
    unit VARCHAR(3) DEFAULT 'EA',
    confidence_lower DECIMAL(15,3),
    confidence_upper DECIMAL(15,3)
);

CREATE INDEX idx_forecast_material ON forecast_plans(material, plant);
CREATE INDEX idx_forecast_periods ON forecast_periods(plan_id);
