-- RBAC Service Initial Schema
-- Roles and Permissions tables

-- Roles table
CREATE TABLE IF NOT EXISTS roles (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT DEFAULT '',
    parent_id VARCHAR(36) REFERENCES roles(id),
    tenant_id VARCHAR(36) NOT NULL,
    is_immutable BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_roles_name ON roles(name);
CREATE INDEX idx_roles_tenant ON roles(tenant_id);

-- Permissions table
CREATE TABLE IF NOT EXISTS permissions (
    id VARCHAR(36) PRIMARY KEY,
    code VARCHAR(100) NOT NULL UNIQUE,
    resource VARCHAR(100) NOT NULL,
    action VARCHAR(50) NOT NULL,
    description TEXT DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_permissions_code ON permissions(code);
CREATE INDEX idx_permissions_resource ON permissions(resource);

-- User-Role mapping (references users from auth-service DB via user_id string)
CREATE TABLE IF NOT EXISTS user_roles (
    user_id VARCHAR(36) NOT NULL,
    role_id VARCHAR(36) NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

-- Role-Permission mapping
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id VARCHAR(36) NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id VARCHAR(36) NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- Seed default roles
INSERT INTO roles (id, name, description, tenant_id, is_immutable) VALUES 
('role_super_admin', 'Super Admin', '超级管理员，拥有所有权限', 'default', TRUE),
('role_admin', 'Admin', '系统管理员', 'default', TRUE),
('role_user', 'User', '普通用户', 'default', TRUE)
ON CONFLICT (id) DO NOTHING;

-- Seed default permissions
INSERT INTO permissions (id, code, resource, action, description) VALUES 
-- User Management
('perm_user_read', 'user:read', 'user', 'read', '查看用户'),
('perm_user_write', 'user:write', 'user', 'write', '编辑用户'),
('perm_user_delete', 'user:delete', 'user', 'delete', '删除用户'),
-- Role Management
('perm_role_read', 'role:read', 'role', 'read', '查看角色'),
('perm_role_write', 'role:write', 'role', 'write', '编辑角色'),
('perm_role_delete', 'role:delete', 'role', 'delete', '删除角色'),
-- System
('perm_system_admin', 'system:admin', 'system', 'admin', '系统管理')
ON CONFLICT (id) DO NOTHING;

-- Grant all permissions to Super Admin
INSERT INTO role_permissions (role_id, permission_id) 
SELECT 'role_super_admin', id FROM permissions
ON CONFLICT DO NOTHING;

-- Assign Super Admin role to admin user
INSERT INTO user_roles (user_id, role_id) VALUES 
('00000000-0000-0000-0000-000000000001', 'role_super_admin')
ON CONFLICT DO NOTHING;
