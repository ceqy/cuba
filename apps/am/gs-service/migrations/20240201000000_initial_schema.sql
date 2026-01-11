CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE system_settings (setting_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(), setting_key VARCHAR(100) UNIQUE NOT NULL, setting_value TEXT, description VARCHAR(255), updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW());
CREATE TABLE config_groups (group_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(), group_name VARCHAR(100) UNIQUE NOT NULL, description VARCHAR(255));
