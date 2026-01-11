CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE incidents (incident_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(), incident_code VARCHAR(20) UNIQUE NOT NULL, category VARCHAR(30), title VARCHAR(255), description TEXT, location VARCHAR(100), incident_datetime TIMESTAMPTZ, reported_by VARCHAR(50), status VARCHAR(20) DEFAULT 'REPORTED', created_at TIMESTAMPTZ NOT NULL DEFAULT NOW());
CREATE TABLE findings (finding_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(), incident_id UUID NOT NULL REFERENCES incidents(incident_id), investigator VARCHAR(50), description TEXT, root_cause TEXT, corrective_actions TEXT);
CREATE INDEX idx_incidents_status ON incidents(status);
