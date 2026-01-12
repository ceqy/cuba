-- OAuth Service Initial Schema
-- OAuth2 clients and tokens

-- OAuth Clients
CREATE TABLE IF NOT EXISTS oauth_clients (
    client_id VARCHAR(64) PRIMARY KEY,
    client_secret VARCHAR(255) NOT NULL,
    name VARCHAR(100) NOT NULL,
    redirect_uris TEXT[] DEFAULT '{}',
    grant_types TEXT[] DEFAULT '{authorization_code}',
    scopes TEXT[] DEFAULT '{openid}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Authorization Codes
CREATE TABLE IF NOT EXISTS oauth_authorization_codes (
    code VARCHAR(255) PRIMARY KEY,
    client_id VARCHAR(64) NOT NULL REFERENCES oauth_clients(client_id),
    user_id VARCHAR(36) NOT NULL,
    redirect_uri TEXT NOT NULL,
    scope TEXT DEFAULT '',
    code_challenge VARCHAR(255),
    code_challenge_method VARCHAR(10),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Refresh Tokens
CREATE TABLE IF NOT EXISTS oauth_refresh_tokens (
    token VARCHAR(255) PRIMARY KEY,
    client_id VARCHAR(64) NOT NULL REFERENCES oauth_clients(client_id),
    user_id VARCHAR(36) NOT NULL,
    scope TEXT DEFAULT '',
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_oauth_codes_client ON oauth_authorization_codes(client_id);
CREATE INDEX idx_oauth_refresh_client ON oauth_refresh_tokens(client_id);
