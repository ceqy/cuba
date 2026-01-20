# CUBA ERP 测试数据问题总结

## 问题描述

在 Swagger UI 中测试接口时发现没有数据返回。

## 问题分析

### 1. ✅ 数据库状态 - 正常
- PostgreSQL 运行正常
- 数据库已创建: `cuba_iam`, `cuba_fi_gl`, `cuba_fi_ap`, `cuba_fi_ar`, `cuba_fi_coa`
- 表结构已创建
- 已有初始用户数据

### 2. ✅ Auth Service - 正常
- 用户注册功能正常
- 用户登录功能正常
- Token 生成正常

**测试账号:**
```
用户名: demo_user
密码: Demo123456
租户: default
```

### 3. ✅ RBAC Service - 正常
- 角色创建功能正常
- 数据库中已有初始角色: `Super Admin`, `Admin`, `User`
- 成功创建测试角色: `finance_manager`

### 4. ⚠️ GL Service - 部分问题

**问题 1: Envoy 端口配置错误**
- 原配置: `cuba-gl-service:50060`
- 实际端口: `cuba-gl-service:50052`
- **已修复**: 修改 Envoy 配置为正确端口

**问题 2: 创建会计分录返回错误**
```json
{
  "code": 3,
  "message": "Missing header",
  "details": []
}
```

**原因分析:**
- gRPC 错误码 3 = INVALID_ARGUMENT
- 可能缺少必需的请求头或字段
- 需要检查 GL Service 的具体实现

## 当前状态

### ✅ 可用功能
1. **用户认证**
   - ✅ 注册新用户
   - ✅ 用户登录
   - ✅ 获取 Token
   - ✅ 刷新 Token

2. **角色权限管理**
   - ✅ 创建角色
   - ✅ 查看角色列表 (数据库中有数据)
   - ⚠️ 列表接口返回 `grpc-status: 12` (UNIMPLEMENTED)

3. **会计分录**
   - ⚠️ 创建分录返回 "Missing header" 错误
   - 数据库表已创建,但无数据

### ⚠️ 需要修复的问题

1. **RBAC Service - ListRoles 接口**
   - 错误: `grpc-status: 12` (UNIMPLEMENTED)
   - 可能原因: 接口未实现或路由配置问题

2. **GL Service - CreateJournalEntry 接口**
   - 错误: "Missing header"
   - 需要检查必需的请求头或字段

## 测试数据

### 已创建的数据

#### 用户
```sql
SELECT username, email FROM users;
```
| username | email |
|----------|-------|
| admin | admin@cuba.local |
| testuser | test@example.com |
| demo_user | demo@cuba.local |
| testuser2 | testuser2@example.com |

#### 角色
```sql
SELECT name, description FROM roles;
```
| name | description |
|------|-------------|
| Super Admin | 超级管理员，拥有所有权限 |
| Admin | 系统管理员 |
| User | 普通用户 |
| finance_manager | 财务经理角色 |

#### 会计分录
```sql
SELECT COUNT(*) FROM journal_entries;
```
结果: 0 条 (因为创建接口有问题)

## 解决方案

### 方案 1: 使用数据库直接插入测试数据

创建 SQL 脚本直接插入测试数据:

```sql
-- 插入测试会计分录
INSERT INTO journal_entries (
  id, company_code, document_number, document_date,
  posting_date, document_type, reference, header_text,
  status, tenant_id, created_by, created_at, updated_at
) VALUES (
  gen_random_uuid(),
  '1000',
  'JE-2026-001',
  '2026-01-20',
  '2026-01-20',
  'SA',
  'TEST-001',
  '测试销售收入',
  'DRAFT',
  'default',
  '4c7c020c-e412-45f8-81d9-b043969fe0be',
  NOW(),
  NOW()
);
```

### 方案 2: 修复 GL Service 接口

检查 GL Service 的实现,确认:
1. 必需的请求头有哪些
2. 请求体的字段是否完整
3. 数据验证逻辑是否正确

### 方案 3: 使用 gRPC 客户端直接测试

绕过 Envoy 网关,直接使用 gRPC 客户端测试:

```bash
grpcurl -plaintext \
  -d '{
    "company_code": "1000",
    "document_date": "2026-01-20",
    "posting_date": "2026-01-20",
    "document_type": "SA",
    "line_items": [...]
  }' \
  localhost:50060 \
  fi.gl.v1.GlJournalEntryService/CreateJournalEntry
```

## 临时解决方案: 直接插入测试数据

运行以下脚本创建测试数据:

```bash
#!/bin/bash

# 连接到数据库并插入测试数据
docker exec cuba-postgres psql -U postgres -d cuba_fi_gl << 'EOF'

-- 插入测试会计分录
INSERT INTO journal_entries (
  id, company_code, document_number, document_date,
  posting_date, document_type, reference, header_text,
  status, tenant_id, created_by, created_at, updated_at
) VALUES
(
  gen_random_uuid(),
  '1000',
  'JE-2026-001',
  '2026-01-20',
  '2026-01-20',
  'SA',
  'TEST-001',
  '测试销售收入',
  'DRAFT',
  'default',
  '4c7c020c-e412-45f8-81d9-b043969fe0be',
  NOW(),
  NOW()
),
(
  gen_random_uuid(),
  '1000',
  'JE-2026-002',
  '2026-01-20',
  '2026-01-20',
  'KR',
  'TEST-002',
  '测试采购成本',
  'POSTED',
  'default',
  '4c7c020c-e412-45f8-81d9-b043969fe0be',
  NOW(),
  NOW()
);

-- 查看插入的数据
SELECT id, document_number, document_type, status FROM journal_entries;

EOF
```

## 在 Swagger UI 中测试

### 1. 访问 Swagger UI
```
http://localhost:8081
```

### 2. 选择 API 文档
选择 **"CUBA ERP - 统一API"**

### 3. 认证
1. 点击右上角 **"Authorize"** 按钮
2. 使用测试账号登录:
   ```
   POST /api/v1/auth/login
   {
     "username": "demo_user",
     "password": "Demo123456",
     "tenant_id": "default"
   }
   ```
3. 复制返回的 `access_token`
4. 在 Authorization 对话框中输入: `Bearer <access_token>`

### 4. 测试可用的接口

#### ✅ 用户管理
- `POST /api/v1/auth/register` - 注册新用户
- `POST /api/v1/auth/login` - 用户登录
- `POST /api/v1/auth/current-user` - 获取当前用户信息
- `POST /api/v1/auth/perm-codes` - 获取权限码

#### ✅ 角色管理
- `POST /api/v1/rbac/roles` - 创建角色 (已测试成功)
- ⚠️ `POST /api/v1/rbac/roles/list` - 列出角色 (接口未实现)

#### ⚠️ 会计分录
- ⚠️ `POST /api/v1/finance/gl/journal-entries` - 创建分录 (Missing header 错误)
- 其他接口未测试

## 建议

### 短期解决方案
1. **使用 SQL 直接插入测试数据** - 可以立即在 Swagger UI 中看到数据
2. **专注测试已实现的接口** - Auth Service 功能完整
3. **记录未实现的接口** - 等待后端开发完成

### 长期解决方案
1. **完善 RBAC Service** - 实现 ListRoles 等查询接口
2. **修复 GL Service** - 解决 "Missing header" 问题
3. **添加接口文档** - 在 OpenAPI 中标注哪些接口已实现
4. **添加健康检查** - 修复服务的 unhealthy 状态
5. **完善错误处理** - 返回更详细的错误信息

## 总结

### 当前可用功能
- ✅ Swagger UI 部署成功
- ✅ Token 共享问题已解决
- ✅ Auth Service 完全可用
- ✅ RBAC Service 部分可用 (创建角色)
- ⚠️ GL Service 需要修复

### 主要问题
1. 部分 gRPC 接口未实现或有 bug
2. 需要更详细的错误信息来调试
3. 建议使用 SQL 直接插入测试数据作为临时方案

### 下一步
1. 创建 SQL 测试数据脚本
2. 联系后端开发团队修复接口问题
3. 完善 API 文档,标注接口状态
