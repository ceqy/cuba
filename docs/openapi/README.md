# CUBA ERP OpenAPI 文档

## 文档结构

### 统一 API 文档
- **cuba-erp-api.yaml**: 包含所有服务的统一 API 文档
  - 优点: 只需登录一次,所有接口共享认证 Token
  - 推荐用于日常开发和测试

### 独立服务文档
位于 `splits/` 目录:
- **auth-service.yaml**: 认证服务 API
- **rbac-service.yaml**: 角色权限服务 API
- **gl-service.yaml**: 总账服务 API
- **ap-service.yaml**: 应付账款服务 API (占位符)
- **ar-service.yaml**: 应收账款服务 API (占位符)
- **coa-service.yaml**: 会计科目表服务 API (占位符)

## API 路径规范

所有 HTTP API 路径遵循以下规范:

### IAM 服务
- Auth Service: `/api/v1/auth/*`
- RBAC Service: `/api/v1/rbac/*`
- OAuth Service: `/api/v1/oauth2/*`

### Finance 服务
- GL Service: `/api/v1/finance/gl/*`
- AP Service: `/api/v1/finance/arap/*`
- AR Service: `/api/v1/finance/ar/*`
- COA Service: `/api/v1/finance/coa/*`

## 认证方式

所有需要认证的接口使用 Bearer Token:

```bash
Authorization: Bearer <access_token>
```

### 获取 Token

1. 调用登录接口:
```bash
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "password",
  "tenant_id": "default"
}
```

2. 响应中获取 access_token:
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

3. 在后续请求中使用:
```bash
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

## Swagger UI 使用

### 本地开发

1. 使用统一 API 文档 (推荐):
```bash
# 启动 Swagger UI
docker run -p 8081:8080 \
  -e PERSIST_AUTHORIZATION=true \
  -v $(pwd)/docs/openapi/cuba-erp-api.yaml:/usr/share/nginx/html/openapi.yaml \
  swaggerapi/swagger-ui
```

访问: http://localhost:8081

2. 使用多个独立文档:
```bash
docker run -p 8081:8080 \
  -e PERSIST_AUTHORIZATION=true \
  -e URLS="[{url: '/auth.yaml', name: 'Auth'}, {url: '/rbac.yaml', name: 'RBAC'}, {url: '/gl.yaml', name: 'GL'}]" \
  -v $(pwd)/docs/openapi/splits:/usr/share/nginx/html \
  swaggerapi/swagger-ui
```

### Kubernetes 部署

1. 创建 ConfigMap:
```bash
kubectl apply -f deploy/k8s/infra/swagger-configmap.yaml
```

2. 部署 Swagger UI:
```bash
kubectl apply -f deploy/k8s/infra/swagger.yaml
```

3. 访问:
```
http://<your-domain>/swagger/
```

## 解决 Token 共享问题

### 问题描述
在 Swagger UI 中切换不同的 API 定义时,Bearer Token 会丢失,需要重新登录。

### 解决方案

**方案 1: 使用统一 API 文档 (推荐)**
- 使用 `cuba-erp-api.yaml` 统一文档
- 所有服务的接口都在一个文档中
- 只需登录一次,Token 在所有接口间共享

**方案 2: 浏览器插件**
- 使用 ModHeader 等浏览器插件
- 设置全局的 Authorization header
- 所有请求自动携带 Token

**方案 3: 自定义 Swagger UI**
- 修改 Swagger UI 的初始化配置
- 使用 localStorage 跨文档共享 Token
- 需要自定义 HTML 页面

## 测试示例

### 1. 登录获取 Token
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "admin123",
    "tenant_id": "default"
  }'
```

### 2. 创建角色
```bash
curl -X POST http://localhost:8080/api/v1/rbac/roles \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your_token>" \
  -d '{
    "name": "finance_manager",
    "description": "财务经理",
    "tenant_id": "default",
    "permissions": ["gl.journal_entry.create", "gl.journal_entry.post"]
  }'
```

### 3. 创建会计分录
```bash
curl -X POST http://localhost:8080/api/v1/finance/gl/journal-entries \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your_token>" \
  -d '{
    "company_code": "1000",
    "document_date": "2026-01-20",
    "posting_date": "2026-01-20",
    "document_type": "SA",
    "reference": "INV-001",
    "header_text": "销售收入",
    "line_items": [
      {
        "account": "110000",
        "debit_credit": "D",
        "amount": 11300,
        "currency": "CNY",
        "text": "应收账款"
      },
      {
        "account": "600000",
        "debit_credit": "C",
        "amount": 10000,
        "currency": "CNY",
        "text": "主营业务收入"
      }
    ],
    "post_immediately": true
  }'
```

## 更新日志

### 2026-01-20
- ✅ 修正所有 OpenAPI 文档的路径格式
  - 从 gRPC 服务名格式改为 HTTP REST 路径
  - 例: `/iam.auth.v1.AuthService/Login` → `/api/v1/auth/login`
- ✅ 更新 server URL
  - 从 `grpc://localhost:50051` 改为 `http://localhost:8080`
- ✅ 创建统一 API 文档 `cuba-erp-api.yaml`
- ✅ 更新 Swagger UI 配置,支持统一认证
- ✅ 创建 Kubernetes ConfigMap 配置

## 注意事项

1. **路径格式**: 所有 API 路径必须以 `/api/v1/` 开头
2. **认证**: 除了登录、注册等公开接口,其他接口都需要 Bearer Token
3. **租户**: 多租户系统,大部分请求需要提供 `tenant_id`
4. **日期格式**: 使用 ISO 8601 格式 `YYYY-MM-DD`
5. **币种**: 默认使用 CNY (人民币)

## 相关文档

- [Envoy Gateway 配置](../../deploy/envoy/envoy.yaml)
- [gRPC Proto 定义](../../proto/)
- [API 开发指南](../development/api-guide.md)
