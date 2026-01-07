-- User Sessions
CREATE TABLE user_sessions (
    session_id VARCHAR(64) PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_name VARCHAR(255) NOT NULL DEFAULT 'Unknown',
    ip_address VARCHAR(45) NOT NULL DEFAULT '0.0.0.0',
    location VARCHAR(255),
    last_active TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);

-- OAuth2 Clients
CREATE TABLE oauth2_clients (
    client_id VARCHAR(255) PRIMARY KEY,
    client_secret_hash VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    redirect_uris TEXT[] NOT NULL DEFAULT '{}',
    grant_types TEXT[] NOT NULL DEFAULT '{}',
    scopes TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
