-- Auth Service Initial Schema
-- Users table for authentication

CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(36) PRIMARY KEY,
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    tenant_id VARCHAR(36) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_tenant ON users(tenant_id);

-- Seed super admin user (password: admin123)
INSERT INTO users (id, username, email, password_hash, tenant_id) VALUES 
('00000000-0000-0000-0000-000000000001', 'admin', 'admin@cuba.local', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/X4.qG6zxnA6TZQ.ZK', 'default')
ON CONFLICT (id) DO NOTHING;
