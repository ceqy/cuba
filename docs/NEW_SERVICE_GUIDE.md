# 新服务创建指南 (Golden Template)

> 本文档记录了如何按照项目"黄金模版"标准创建新的 gRPC 服务并生成 OpenAPI 文档。

## 📋 快速开始

### 1. 创建 Proto 定义

在 `protos/<service_path>/<service_name>.proto` 中添加以下标准配置：

```protobuf
syntax = "proto3";

package your.service;
option go_package = "github.com/yourproject/your/service";

import "google/protobuf/timestamp.proto";
import "google/api/annotations.proto";
import "protoc-gen-openapiv2/options/openapiv2.proto";
import "common/common.proto";

// OpenAPI 全局配置 (Golden Standard)
option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_swagger) = {
  info: {
    title: "Your Service API"
    version: "5.31.0"  // 【重要】统一版本号，请勿随意修改
    description: "服务功能描述"
    contact: {
      name: "Your Team"
      email: "team@yourproject.com"
    }
  }
  host: "localhost:8080"
  base_path: "/"
  schemes: HTTP
  schemes: HTTPS
  consumes: "application/json"
  produces: "application/json"
  security_definitions: {
    security: {
      key: "bearer_auth"
      value: {
        type: TYPE_API_KEY
        in: IN_HEADER
        name: "Authorization"
        description: "Bearer token for authentication"
      }
    }
  }
  security: {
    security_requirement: {
      key: "bearer_auth"
      value: {}
    }
  }
};

service YourService {
  option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_tag) = {
    description: "服务描述"
    external_docs: {
      url: "https://yourproject.com/docs/your-service";
      description: "详细文档";
    }
  };

  // 定义你的 RPC 方法
  rpc YourMethod(YourRequest) returns (YourResponse) {
    option (google.api.http) = {
      post: "/api/v1/your-service/your-method"
      body: "*"
    };
    option (grpc.gateway.protoc_gen_openapiv2.options.openapiv2_operation) = {
      tags: "Your Tag"
      summary: "方法描述"
    };
  }
}
```

### 2. 生成 OpenAPI 文档

```bash
./scripts/gen_openapi.sh <service_path>
```

**示例：**
```bash
./scripts/gen_openapi.sh finance/gl
./scripts/gen_openapi.sh auth
./scripts/gen_openapi.sh finance/ar_ap
```

### 3. 更新 Swagger UI 配置

编辑 `scripts/start.sh`，在 Swagger UI 的 `URLS` 环境变量中添加新服务：

```bash
-e URLS="[
  { \"url\": \"/docs/auth/auth_service.openapi3.json\", \"name\": \"Auth Service\" },
  { \"url\": \"/docs/finance/gl/gl_journal_entry.openapi3.json\", \"name\": \"GL Service\" },
  { \"url\": \"/docs/finance/ar_ap/ar_ap.openapi3.json\", \"name\": \"AR/AP Service\" },
  { \"url\": \"/docs/<your_path>/<your_service>.openapi3.json\", \"name\": \"Your Service\" }
]"
```

### 4. 验证

```bash
# 检查生成的文档版本号
grep "\"version\"" docs/<your_path>/<your_service>.openapi3.json

# 重启 Swagger UI
docker rm -f swagger-ui
./scripts/start.sh
```

访问 `http://localhost:8081` 确认新服务出现在下拉列表中。

---

## 🎯 关键要点

### ✅ 必须遵守的规则

1. **OpenAPI 配置必须在文件顶层**（service 定义之前）
2. **版本号统一为 5.31.0**（除非有明确的升级计划）
3. **每个 RPC 方法必须包含**：
   - `google.api.http` 注解（定义 REST 路径）
   - `openapiv2_operation` 注解（定义 Tags 和 Summary）
4. **使用统一脚本生成文档**：`gen_openapi.sh`，不要创建服务专用脚本

### 📁 目录结构

生成的文档会按以下结构组织：

```
docs/
├── auth/
│   ├── auth_service.swagger.json
│   └── auth_service.openapi3.json
└── finance/
    ├── gl/
    │   ├── gl_journal_entry.swagger.json
    │   └── gl_journal_entry.openapi3.json
    └── ar_ap/
        ├── ar_ap.swagger.json
        └── ar_ap.openapi3.json
```

---

## 🔄 版本管理策略

### 固定版本（当前策略）
- **版本号**：`5.31.0`
- **适用场景**：内部项目、快速迭代阶段
- **优点**：简单，前端无需频繁适配
- **缺点**：无法通过版本号识别 API 变更

### 语义化版本（推荐用于生产）
- **Patch (5.31.1)**：Bug 修复、文档更新
- **Minor (5.32.0)**：新增功能（向后兼容）
- **Major (6.0.0)**：破坏性变更（删除字段、重命名接口等）

**修改方式**：手动编辑 `.proto` 文件中的 `version` 字段，然后重新生成文档。

---

## � 版本号说明（重要）

项目中有**三个不同的版本号**，容易混淆，请注意区分：

### 1. Swagger UI 工具版本
```bash
# 在 scripts/start.sh 中定义
swaggerapi/swagger-ui:v5.31.0
```
- **含义**：Swagger UI 网页工具的版本
- **作用**：决定界面功能和性能
- **修改**：编辑 `scripts/start.sh` 中的 Docker 镜像版本

### 2. OpenAPI 规范版本
```json
{
  "openapi": "3.1.0"  // 在生成的 JSON 文件中
}
```
- **含义**：文档格式遵循的标准版本
- **作用**：告诉工具如何解析文档
- **修改**：由 `swagger2openapi` 自动设置，**不要手动改**

### 3. API 服务版本
```json
{
  "info": {
    "version": "5.31.0"  // 你的服务版本
  }
}
```
- **含义**：你的 API 接口的业务版本
- **作用**：标识 API 的迭代版本，用于版本管理
- **修改**：编辑 `.proto` 文件中的 `version` 字段

### 版本号对照表

| 位置 | 版本号示例 | 含义 | 是否可改 |
|------|-----------|------|---------|
| `scripts/start.sh` | `v5.31.0` | Swagger UI 工具版本 | ✅ 可以 |
| JSON `"openapi"` | `"3.1.0"` | OpenAPI 规范版本 | ❌ 不要改 |
| JSON `"info.version"` | `"5.31.0"` | 你的 API 版本 | ✅ 应该改 |

**记忆口诀**：
- Swagger UI 5.31.0 = 展示工具的版本（浏览器界面）
- OpenAPI 3.1.0 = 文档格式的版本（JSON 标准）
- API 5.31.0 = 你的服务的版本（业务逻辑）

---

## �🛠️ 故障排查

### Swagger UI 无法加载服务

**症状**：下拉列表中看不到新服务，或点击后显示 404

**解决方案**：
1. 检查 `start.sh` 中的路径是否正确（注意嵌套目录结构）
2. 确认文档已生成：`ls docs/<your_path>/<your_service>.openapi3.json`
3. 重启 Swagger UI：`docker rm -f swagger-ui && ./scripts/start.sh`

### 生成的文档版本号不对

**症状**：`grep "version" docs/...` 显示的版本号与 `.proto` 不一致

**解决方案**：
```bash
# 清理旧文件并重新生成
rm -rf docs/<your_path>
./scripts/gen_openapi.sh <service_path>
```

### protoc 报错找不到文件

**症状**：`No such file or directory: protos/...`

**解决方案**：
- 确认 `.proto` 文件路径正确
- 检查 `import` 语句中的路径（特别是 `common/common.proto`）
- 确保 `protos/third_party` 目录存在

---

## 📚 参考资料

- **标准定义**：`/Users/x/.gemini/antigravity/brain/.../standards.md`
- **示例 Proto**：
  - `protos/finance/gl/gl_journal_entry.proto`
  - `protos/finance/ar_ap/ar_ap.proto`
  - `protos/auth/auth_service.proto`
- **生成脚本**：`scripts/gen_openapi.sh`

---

**最后更新**：2026-01-09
