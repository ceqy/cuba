CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Surveys
CREATE TABLE surveys (
    survey_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR(255) NOT NULL,
    target_audience VARCHAR(100),
    status VARCHAR(20) DEFAULT 'DRAFT',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Survey Responses  
CREATE TABLE survey_responses (
    response_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    survey_id UUID NOT NULL REFERENCES surveys(survey_id),
    employee_id VARCHAR(20) NOT NULL,
    answers JSONB,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Recognitions
CREATE TABLE recognitions (
    recognition_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    giver_employee_id VARCHAR(20) NOT NULL,
    receiver_employee_id VARCHAR(20) NOT NULL,
    message TEXT,
    company_value VARCHAR(100),
    status VARCHAR(20) DEFAULT 'ACTIVE',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_rec_receiver ON recognitions(receiver_employee_id);
