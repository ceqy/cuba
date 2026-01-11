-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Candidates
CREATE TABLE candidates (
    candidate_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    phone VARCHAR(20),
    resume_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Job Applications
CREATE TABLE job_applications (
    application_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    requisition_id VARCHAR(20) NOT NULL,
    requisition_title VARCHAR(255),
    candidate_id UUID NOT NULL REFERENCES candidates(candidate_id),
    status VARCHAR(20) DEFAULT 'SUBMITTED', -- SUBMITTED, SCREENING, INTERVIEW, OFFER, HIRED, REJECTED
    application_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Interview Schedules
CREATE TABLE interview_schedules (
    interview_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    application_id UUID NOT NULL REFERENCES job_applications(application_id),
    interview_type VARCHAR(20) NOT NULL, -- PHONE, VIDEO, ONSITE
    scheduled_time TIMESTAMPTZ NOT NULL,
    interviewer_id VARCHAR(20),
    location VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_app_candidate ON job_applications(candidate_id);
CREATE INDEX idx_app_requisition ON job_applications(requisition_id);
