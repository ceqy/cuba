# CUBA ERP Swagger UI 测试指南

## 🎯 快速开始

### 1. 访问 Swagger UI
打开浏览器访问: **http://localhost:8081**

### 2. 选择 API 文档
在右上角下拉菜单中选择: **"CUBA ERP - 统一API"**

### 3. 登录获取 Token

#### 方法 A: 使用现有测试账号
```json
POST /api/v1/auth/login
{
  "username": "demo_user",
  "password": "Demo123456",
  "tenant_id": "default"
}
```

#### 方法 B: 注册新账号
```json
POST /api/v1/auth/register
{
  "username": "your_username",
  "password": "YourPassword123",
  "email": "your@email.com",
  "tenant_id": "default"
}
```

### 4. 设置认证
1. 点击右上角 **"Authorize"** 按钮
2. 在弹出的对话框中输入: `Bearer <your_access_token>`
3. 点击 "Authorize" 确认
4. 点击 "Close" 关闭对话框

### 5. 开始测试
现在可以测试所有接口了! Token 会在所有接口间共享,无需重新登录。

## ✅ 已验证可用的功能

### Auth Service (认证服务)
| 接口 | 方法 | 路径 | 状态 |
|------|------|------|------|
| 用户注册 | POST | `/api/v1/auth/register` | ✅ 正常 |
| 用户登录 | POST | `/api/v1/auth/login` | ✅ 正常 |
| 刷新Token | POST | `/api/v1/auth/refresh-token` | ✅ 正常 |
| 获取当前用户 | POST | `/api/v1/auth/current-user` | ✅ 正常 |
| 获取权限码 | POST | `/api/v1/auth/perm-codes` | ✅ 正常 |

### RBAC Service (角色权限服务)
| 接口 | 方法 | 路径 | 状态 |
|------|------|------|------|
| 创建角色 | POST | `/api/v1/rbac/roles` | ✅ 正常 |
| 列出角色 | POST | `/api/v1/rbac/roles/list` | ⚠️ 未实现 |
| 检查权限 | POST | `/api/v1/rbac/permissions/check` | ⚠️ 未测试 |

### GL Service (总账服务)
| 接口 | 方法 | 路径 | 状态 |
|------|------|------|------|
| 创建会计分录 | POST | `/api/v1/finance/gl/journal-entries` | ⚠️ 有问题 |
| 查询分录列表 | POST | `/api/v1/finance/gl/journal-entries/list` | ⚠️ 未测试 |
| 获取分录详情 | POST | `/api/v1/finance/gl/journal-entries/get` | ⚠️ 未测试 |

## 📊 测试数据

### 用户数据
| 用户名 | 密码 | 邮箱 | 租户 |
|--------|------|------|------|
| demo_user | Demo123456 | demo@cuba.local | default |
| testuser2 | Test123456 | testuser2@example.com | default |

### 角色数据
| 角色名 | 描述 | 状态 |
|--------|------|------|
| Super Admin | 超级管理员 | 系统预置 |
| Admin | 系统管理员 | 系统预置 |
| User | 普通用户 | 系统预置 |
| finance_manager | 财务经理角色 | 测试创建 |

### 会计分录数据
⚠️ **注意**: 由于 GL Service 的创建接口有问题,目前数据库中没有会计分录数据。

**临时解决方案**: 可以使用 SQL 脚本直接插入测试数据:
```bash
./scripts/insert-test-data.sh
```

## 🔧 已知问题

### 1. Token 共享问题 ✅ 已解决
**问题**: 切换不同的 API 定义时,Token 会丢失
**解决方案**: 使用统一 API 文档 "CUBA ERP - 统一API"

### 2. RBAC ListRoles 接口未实现
**问题**: 调用 `/api/v1/rbac/roles/list` 返回 `grpc-status: 12` (UNIMPLEMENTED)
**影响**: 无法通过 API 查询角色列表
**临时方案**: 直接查询数据库

### 3. GL CreateJournalEntry 接口错误
**问题**: 调用 `/api/v1/finance/gl/journal-entries` 返回 "Missing header"
**影响**: 无法通过 API 创建会计分录
**临时方案**: 使用 SQL 脚本直接插入数据

### 4. 服务健康检查失败
**问题**: 所有后端服务显示 `unhealthy` 状态
**影响**: 不影响功能,服务仍然可以正常响应
**建议**: 检查健康检查端点配置

## 📝 测试示例

### 示例 1: 完整的用户注册和登录流程

```bash
# 1. 注册新用户
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "test_user",
    "password": "Test123456",
    "email": "test@example.com",
    "tenant_id": "default"
  }'

# 2. 登录获取 Token
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "test_user",
    "password": "Test123456",
    "tenant_id": "default"
  }'

# 3. 使用 Token 获取当前用户信息
curl -X POST http://localhost:8080/api/v1/auth/current-user \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your_token>" \
  -d '{}'
```

### 示例 2: 创建角色

```bash
curl -X POST http://localhost:8080/api/v1/rbac/roles \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your_token>" \
  -d '{
    "name": "accountant",
    "description": "会计角色",
    "tenant_id": "default",
    "permissions": [
      "gl.journal_entry.view",
      "gl.journal_entry.list"
    ]
  }'
```

## 🛠️ 故障排除

### 问题: 无法访问 Swagger UI
**检查**:
```bash
docker ps | grep swagger
curl http://localhost:8081
```

### 问题: API 返回 "no healthy upstream"
**检查**:
```bash
# 检查 Envoy 网关
docker ps | grep envoy
docker logs cuba-envoy --tail 20

# 检查后端服务
docker ps | grep cuba-
docker restart cuba-envoy
```

### 问题: 登录返回 "Invalid credentials"
**解决**:
- 确认用户名和密码正确
- 确认租户 ID 为 "default"
- 或者注册新用户

### 问题: Token 过期
**解决**:
- 重新登录获取新 Token
- 或使用 refresh_token 刷新

## 📚 相关文档

- **OpenAPI 文档**: `docs/openapi/cuba-erp-api.yaml`
- **部署测试报告**: `docs/openapi/DEPLOYMENT_TEST.md`
- **数据问题总结**: `docs/openapi/DATA_ISSUES.md`
- **使用说明**: `docs/openapi/README.md`

## 🔄 更新日志

### 2026-01-20
- ✅ 修正所有 OpenAPI 文档路径 (从 gRPC 格式改为 HTTP REST)
- ✅ 创建统一 API 文档,解决 Token 共享问题
- ✅ 修正 Envoy 网关配置 (服务地址和端口)
- ✅ 部署 Swagger UI 到 Docker
- ✅ 创建测试数据初始化脚本
- ✅ 验证 Auth Service 和 RBAC Service 功能
- ⚠️ 发现 GL Service 接口问题,待修复

## 💡 最佳实践

### 1. 使用统一 API 文档
始终选择 "CUBA ERP - 统一API",避免 Token 共享问题。

### 2. 保存 Token
登录后保存 Token,可以在多个工具中使用:
- Swagger UI
- Postman
- curl 命令
- 自动化测试脚本

### 3. 测试顺序
建议按以下顺序测试:
1. 注册/登录 (Auth Service)
2. 创建角色 (RBAC Service)
3. 分配权限
4. 创建业务数据 (GL Service 等)

### 4. 错误处理
遇到错误时:
1. 检查请求体格式是否正确
2. 确认 Token 是否有效
3. 查看服务日志: `docker logs <service_name>`
4. 检查数据库数据: `docker exec cuba-postgres psql ...`

## 🎉 总结

### 成功完成
- ✅ Swagger UI 部署成功
- ✅ Token 共享问题已解决
- ✅ Auth Service 完全可用
- ✅ RBAC Service 部分可用
- ✅ 创建了完整的测试文档和脚本

### 待改进
- ⚠️ 修复 RBAC ListRoles 接口
- ⚠️ 修复 GL CreateJournalEntry 接口
- ⚠️ 添加更多测试数据
- ⚠️ 完善错误信息
- ⚠️ 修复服务健康检查

### 下一步
1. 联系后端团队修复接口问题
2. 使用 SQL 脚本插入测试数据
3. 完善 API 文档,标注接口状态
4. 添加更多测试用例

---

**需要帮助?**
- 查看文档: `docs/openapi/`
- 查看日志: `docker logs <service_name>`
- 运行测试脚本: `./scripts/init-test-data.sh`
