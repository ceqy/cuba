-- API Key Management
-- 2026-01-07

CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    prefix VARCHAR(20) NOT NULL, -- 如 "ak_1a2b3" 用于显示
    key_hash VARCHAR(255) NOT NULL, -- 存储哈希后的完整 Key
    scopes TEXT[], -- 权限范围
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id VARCHAR(50),
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON api_keys(prefix);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash);
