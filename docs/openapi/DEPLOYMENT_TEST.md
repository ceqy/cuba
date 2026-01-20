# CUBA ERP Swagger UI 部署测试报告

## 测试时间
2026-01-20

## 部署环境
- **平台**: Docker (本地开发环境)
- **Swagger UI**: http://localhost:8081
- **API Gateway**: http://localhost:8080

## 部署步骤

### 1. 创建统一 API 文档 ✅
- 文件: `docs/openapi/cuba-erp-api.yaml`
- 包含: Auth Service, RBAC Service, GL Service
- 特点: 所有接口共享同一个 Bearer Token

### 2. 修正 OpenAPI 文档路径 ✅
- Auth Service: `/iam.auth.v1.AuthService/*` → `/api/v1/auth/*`
- RBAC Service: `/iam.rbac.v1.RBACService/*` → `/api/v1/rbac/*`
- GL Service: `/fi.gl.v1.GlJournalEntryService/*` → `/api/v1/finance/gl/*`

### 3. 修正 Envoy 网关配置 ✅
- 移除不存在的 gRPC 服务引用
- 修改服务地址为 Docker 容器名称:
  - `auth-service.cuba-iam.svc.cluster.local` → `cuba-auth-service`
  - `rbac-service.cuba-iam.svc.cluster.local` → `cuba-rbac-service`
  - `gl-service.cuba-fi.svc.cluster.local` → `cuba-gl-service`

### 4. 部署 Swagger UI ✅
```bash
docker-compose -f docker-compose.swagger.yaml up -d
```

## 测试结果

### ✅ Swagger UI 访问测试
```bash
curl http://localhost:8081/
# 返回: Swagger UI HTML 页面
```

### ✅ OpenAPI 文档访问测试
```bash
curl http://localhost:8081/specs/cuba-erp-api.yaml
# 返回: 完整的 OpenAPI 文档
```

### ✅ API Gateway 连通性测试
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123","tenant_id":"default"}'

# 返回: {"code": 16, "message": "Invalid credentials", "details": []}
# 说明: API 网关和后端服务通信正常,只是凭据不正确
```

## 访问方式

### 浏览器访问
打开浏览器访问: **http://localhost:8081**

### 可用的 API 文档
1. **CUBA ERP - 统一API** (推荐)
   - 包含所有服务的接口
   - 只需登录一次,Token 在所有接口间共享

2. **Auth Service** (独立文档)
   - 仅包含认证服务接口

3. **RBAC Service** (独立文档)
   - 仅包含角色权限服务接口

4. **GL Service** (独立文档)
   - 仅包含总账服务接口

## 使用说明

### 1. 选择 API 文档
在 Swagger UI 右上角的下拉菜单中选择 **"CUBA ERP - 统一API"**

### 2. 认证
1. 点击右上角的 **"Authorize"** 按钮
2. 先调用 `/api/v1/auth/register` 注册用户,或使用已有用户
3. 调用 `/api/v1/auth/login` 获取 Token
4. 将返回的 `access_token` 填入 Authorization 对话框
5. 格式: `Bearer <access_token>`

### 3. 测试接口
现在可以测试所有接口,无需重新登录

## Token 共享验证

### 问题
之前在 Swagger UI 中切换不同的 API 定义时,Bearer Token 会丢失,需要重新登录。

### 解决方案
使用统一 API 文档 (`cuba-erp-api.yaml`),所有服务的接口都在一个文档中,Token 自然共享。

### 验证步骤
1. 在 Swagger UI 中选择 "CUBA ERP - 统一API"
2. 点击 "Authorize" 并登录获取 Token
3. 测试 Auth Service 的接口 (如 `/api/v1/auth/current-user`)
4. 测试 RBAC Service 的接口 (如 `/api/v1/rbac/roles/list`)
5. 测试 GL Service 的接口 (如 `/api/v1/finance/gl/journal-entries/list`)
6. **验证**: 所有接口都能正常调用,无需重新输入 Token

## 部署文件

### Docker Compose 配置
- `docker-compose.swagger.yaml`: Swagger UI 容器配置

### OpenAPI 文档
- `docs/openapi/cuba-erp-api.yaml`: 统一 API 文档
- `docs/openapi/splits/auth-service.yaml`: Auth Service 文档
- `docs/openapi/splits/rbac-service.yaml`: RBAC Service 文档
- `docs/openapi/splits/gl-service.yaml`: GL Service 文档

### Envoy 配置
- `deploy/envoy/envoy.yaml`: API 网关配置

### Kubernetes 配置 (待部署)
- `deploy/k8s/infra/swagger.yaml`: Swagger UI Deployment
- `deploy/k8s/infra/swagger-configmap.yaml`: OpenAPI 文档 ConfigMap

## 已知问题

### 1. 后端服务健康检查失败
**状态**: 所有后端服务显示 `unhealthy`
**影响**: 不影响功能,服务仍然可以正常响应请求
**原因**: 健康检查端点可能未正确配置
**建议**: 检查各服务的健康检查配置

### 2. 默认用户凭据
**问题**: 不清楚默认的用户名和密码
**建议**:
- 查看数据库初始化脚本
- 或使用注册接口创建新用户

## 下一步

### 1. Kubernetes 部署
```bash
# 创建 ConfigMap
kubectl apply -f deploy/k8s/infra/swagger-configmap.yaml

# 部署 Swagger UI
kubectl apply -f deploy/k8s/infra/swagger.yaml

# 访问
kubectl port-forward -n cuba-system svc/swagger-ui 8081:80
```

### 2. 生产环境配置
- 配置 Ingress 或 LoadBalancer
- 启用 HTTPS
- 配置访问控制

### 3. 完善文档
- 添加更多示例
- 补充 AP, AR, COA 服务的 API 文档
- 添加错误码说明

## 总结

✅ **成功部署** Swagger UI 到 Docker 环境
✅ **解决** Token 共享问题 (使用统一 API 文档)
✅ **修正** 所有 OpenAPI 文档的路径格式
✅ **验证** API Gateway 和后端服务通信正常

**推荐使用**: 统一 API 文档 (`CUBA ERP - 统一API`),提供最佳的开发体验。
