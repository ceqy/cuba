-- Migration: Create policies tables for ABAC
-- Created: 2026-01-07

-- Policies table (stores ABAC policy definitions)
CREATE TABLE IF NOT EXISTS policies (
    policy_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    version VARCHAR(50) NOT NULL DEFAULT '1.0',
    statements JSONB NOT NULL DEFAULT '[]'::jsonb,  -- Array of Statement objects
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Role-Policy association (many-to-many)
CREATE TABLE IF NOT EXISTS role_policies (
    role_id UUID NOT NULL REFERENCES roles(role_id) ON DELETE CASCADE,
    policy_id UUID NOT NULL REFERENCES policies(policy_id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (role_id, policy_id)
);

-- User-Policy association (direct policy attachment to users)
CREATE TABLE IF NOT EXISTS user_policies (
    user_id UUID NOT NULL REFERENCES users(user_id) ON DELETE CASCADE,
    policy_id UUID NOT NULL REFERENCES policies(policy_id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, policy_id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_policies_name ON policies(name);
CREATE INDEX IF NOT EXISTS idx_role_policies_role_id ON role_policies(role_id);
CREATE INDEX IF NOT EXISTS idx_role_policies_policy_id ON role_policies(policy_id);
CREATE INDEX IF NOT EXISTS idx_user_policies_user_id ON user_policies(user_id);
CREATE INDEX IF NOT EXISTS idx_user_policies_policy_id ON user_policies(policy_id);

-- Comments
COMMENT ON TABLE policies IS 'ABAC policy definitions with statements';
COMMENT ON COLUMN policies.statements IS 'JSON array of {effect, actions, resources, conditions}';
