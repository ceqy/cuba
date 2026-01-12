-- roles: Defines abstract roles
CREATE TABLE roles (
    id VARCHAR(50) PRIMARY KEY, -- e.g., 'role_super_admin'
    name VARCHAR(100) NOT NULL,
    description TEXT,
    parent_role_id VARCHAR(50) REFERENCES roles(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- permissions: Defines granular access codes
CREATE TABLE permissions (
    id VARCHAR(50) PRIMARY KEY, -- e.g., 'perm_user_add'
    code VARCHAR(100) NOT NULL UNIQUE, -- 'AC_USER_ADD' (Used by Frontend)
    resource VARCHAR(50) NOT NULL, -- 'iam:user'
    action VARCHAR(50) NOT NULL, -- 'write'
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- user_roles: Many-to-Many User <-> Role
CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id VARCHAR(50) NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, role_id)
);

-- role_permissions: Many-to-Many Role <-> Permission
CREATE TABLE role_permissions (
    role_id VARCHAR(50) NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id VARCHAR(50) NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- Seed Initial Data
INSERT INTO roles (id, name, description) VALUES 
('role_super_admin', '超级管理员', '系统最高权限角色'),
('role_default_user', '普通用户', '默认基础权限角色')
ON CONFLICT DO NOTHING;

INSERT INTO permissions (id, code, resource, action, description) VALUES 
('perm_user_all', 'AC_USER_ALL', 'iam:user', '*', '用户管理全权限'),
('perm_auth_me', 'AC_AUTH_ME', 'iam:auth', 'read', '查看个人信息权限')
ON CONFLICT DO NOTHING;

-- Bind permissions to super admin
INSERT INTO role_permissions (role_id, permission_id)
SELECT 'role_super_admin', id FROM permissions
ON CONFLICT DO NOTHING;

-- Bind existing admin user to super admin role
INSERT INTO user_roles (user_id, role_id)
SELECT id, 'role_super_admin' FROM users WHERE username = 'admin'
ON CONFLICT DO NOTHING;
