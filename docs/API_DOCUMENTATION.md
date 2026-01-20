# CUBA ERP API 文档

> 版本: 1.0.0
> 最后更新: 2026-01-19
> 协议: gRPC

## 目录

- [概述](#概述)
- [认证](#认证)
- [IAM 模块](#iam-模块)
  - [Auth Service](#auth-service)
  - [RBAC Service](#rbac-service)
- [财务模块](#财务模块)
  - [GL Service](#gl-service)
  - [AP Service](#ap-service)
  - [AR Service](#ar-service)
  - [COA Service](#coa-service)
- [通用规范](#通用规范)
- [错误处理](#错误处理)
- [示例代码](#示例代码)

---

## 概述

CUBA ERP 采用微服务架构，所有服务通过 gRPC 协议通信。本文档描述了所有可用的 API 接口。

### 服务端点

| 服务 | 地址 | 端口 | 描述 |
|------|------|------|------|
| Auth Service | localhost | 50051 | 用户认证与授权 |
| RBAC Service | localhost | 50052 | 角色权限管理 |
| GL Service | localhost | 50060 | 总账管理 |
| AP Service | localhost | 50061 | 应付账款管理 |
| AR Service | localhost | 50062 | 应收账款管理 |
| COA Service | localhost | 50065 | 会计科目表管理 |

### 技术栈

- **协议**: gRPC (Protocol Buffers)
- **认证**: JWT (JSON Web Token)
- **传输**: HTTP/2
- **编码**: Protocol Buffers v3

---

## 认证

### JWT Token 认证

大多数 API 需要在请求头中携带 JWT Token：

```
Authorization: Bearer <access_token>
```

### 获取 Token

通过 Auth Service 的 `Login` 方法获取：

```bash
grpcurl -plaintext -d '{
  "username": "your_username",
  "password": "your_password",
  "tenant_id": "default"
}' localhost:50051 iam.auth.v1.AuthService/Login
```

响应：
```json
{
  "accessToken": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "refreshToken": "uuid-string",
  "expiresIn": 86400,
  "tokenType": "Bearer",
  "sessionId": "session-uuid"
}
```

### Token 刷新

Token 过期后使用 `RefreshToken` 方法刷新：

```bash
grpcurl -plaintext -d '{
  "refreshToken": "your_refresh_token"
}' localhost:50051 iam.auth.v1.AuthService/RefreshToken
```

---

## IAM 模块

### Auth Service

**服务名**: `iam.auth.v1.AuthService`
**端口**: 50051

#### 方法列表

##### 1. Register - 用户注册

注册新用户账号。

**请求**:
```protobuf
message RegisterRequest {
  string username = 1;        // 用户名（必填，唯一）
  string password = 2;        // 密码（必填，最少8位）
  string email = 3;           // 邮箱（必填，唯一）
  string tenant_id = 4;       // 租户ID（必填）
  string idempotency_key = 5; // 幂等性键（可选）
}
```

**响应**:
```protobuf
message RegisterResponse {
  string user_id = 1;    // 用户ID
  string username = 2;   // 用户名
  string status = 3;     // 状态: ACTIVE, PENDING, SUSPENDED
}
```

**示例**:
```bash
grpcurl -plaintext -d '{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "SecurePass123!",
  "tenant_id": "default"
}' localhost:50051 iam.auth.v1.AuthService/Register
```

**错误码**:
- `ALREADY_EXISTS`: 用户名或邮箱已存在
- `INVALID_ARGUMENT`: 参数验证失败
- `INTERNAL`: 服务器内部错误

---

##### 2. Login - 用户登录

用户登录并获取访问令牌。

**请求**:
```protobuf
message LoginRequest {
  string username = 1;       // 用户名（必填）
  string password = 2;       // 密码（必填）
  string tenant_id = 3;      // 租户ID（必填）
  string two_factor_code = 4; // 2FA代码（启用2FA时必填）
}
```

**响应**:
```protobuf
message LoginResponse {
  string access_token = 1;   // JWT访问令牌
  string refresh_token = 2;  // 刷新令牌
  int64 expires_in = 3;      // 过期时间（秒）
  string token_type = 4;     // 令牌类型: "Bearer"
  string session_id = 5;     // 会话ID
}
```

**示例**:
```bash
grpcurl -plaintext -d '{
  "username": "john_doe",
  "password": "SecurePass123!",
  "tenant_id": "default"
}' localhost:50051 iam.auth.v1.AuthService/Login
```

---

##### 3. RefreshToken - 刷新令牌

使用刷新令牌获取新的访问令牌。

**请求**:
```protobuf
message RefreshTokenRequest {
  string refresh_token = 1;  // 刷新令牌（必填）
}
```

**响应**:
```protobuf
message RefreshTokenResponse {
  string access_token = 1;   // 新的访问令牌
  string refresh_token = 2;  // 新的刷新令牌
  int64 expires_in = 3;      // 过期时间（秒）
}
```

---

##### 4. Logout - 用户登出

注销当前会话。

**请求**:
```protobuf
message LogoutRequest {
  string session_id = 1;  // 会话ID（可选，默认当前会话）
}
```

**响应**:
```protobuf
message LogoutResponse {
  bool success = 1;       // 是否成功
  string message = 2;     // 消息
}
```

**需要认证**: ✅

---

##### 5. GetCurrentUser - 获取当前用户信息

获取当前登录用户的详细信息。

**请求**: `google.protobuf.Empty`

**响应**:
```protobuf
message GetCurrentUserResponse {
  string user_id = 1;
  string username = 2;
  string email = 3;
  string tenant_id = 4;
  repeated string roles = 5;
  string created_at = 6;
  string updated_at = 7;
}
```

**需要认证**: ✅

---

##### 6. ChangePassword - 修改密码

修改当前用户密码。

**请求**:
```protobuf
message ChangePasswordRequest {
  string old_password = 1;  // 旧密码（必填）
  string new_password = 2;  // 新密码（必填）
}
```

**响应**:
```protobuf
message ChangePasswordResponse {
  bool success = 1;
  string message = 2;
}
```

**需要认证**: ✅

---

##### 7. UpdateProfile - 更新个人资料

更新用户个人资料信息。

**请求**:
```protobuf
message UpdateProfileRequest {
  string email = 1;          // 新邮箱（可选）
  string display_name = 2;   // 显示名称（可选）
  string avatar_url = 3;     // 头像URL（可选）
}
```

**需要认证**: ✅

---

##### 8. Enable2FA - 启用双因素认证

为当前用户启用2FA。

**请求**: `google.protobuf.Empty`

**响应**:
```protobuf
message Enable2FAResponse {
  string secret = 1;         // TOTP密钥
  string qr_code_url = 2;    // 二维码URL
  repeated string backup_codes = 3; // 备份码
}
```

**需要认证**: ✅

---

##### 9. ListUsers - 列出用户（管理员）

列出所有用户（需要管理员权限）。

**请求**:
```protobuf
message ListUsersRequest {
  int32 page = 1;           // 页码（从1开始）
  int32 page_size = 2;      // 每页数量
  string tenant_id = 3;     // 租户ID过滤
  string status = 4;        // 状态过滤
}
```

**需要认证**: ✅
**需要权限**: `users.list`

---

##### 10. GetPermCodes - 获取权限码

获取当前用户的所有权限码（用于前端权限控制）。

**请求**: `google.protobuf.Empty`

**响应**:
```protobuf
message GetPermCodesResponse {
  repeated string perm_codes = 1;  // 权限码列表
}
```

**需要认证**: ✅

---

### RBAC Service

**服务名**: `iam.rbac.v1.RBACService`
**端口**: 50052

#### 方法列表

##### 1. CreateRole - 创建角色

创建新的角色。

**请求**:
```protobuf
message CreateRoleRequest {
  string name = 1;           // 角色名称（必填）
  string description = 2;    // 角色描述
  string tenant_id = 3;      // 租户ID
  repeated string permissions = 4; // 权限列表
}
```

**响应**:
```protobuf
message Role {
  string role_id = 1;
  string name = 2;
  string description = 3;
  repeated string permissions = 4;
  string created_at = 5;
}
```

**需要认证**: ✅
**需要权限**: `roles.create`

---

##### 2. AssignRoleToUser - 分配角色给用户

将角色分配给指定用户。

**请求**:
```protobuf
message AssignRoleToUserRequest {
  string user_id = 1;        // 用户ID（必填）
  string role_id = 2;        // 角色ID（必填）
  string tenant_id = 3;      // 租户ID
}
```

**需要认证**: ✅
**需要权限**: `roles.assign`

---

##### 3. CheckPermissions - 检查权限

检查用户是否拥有指定权限。

**请求**:
```protobuf
message CheckPermissionsRequest {
  string user_id = 1;                // 用户ID
  repeated string permissions = 2;   // 要检查的权限列表
}
```

**响应**:
```protobuf
message CheckPermissionsResponse {
  map<string, bool> results = 1;  // 权限检查结果
}
```

**需要认证**: ✅

---

##### 4. GetUserRoles - 获取用户角色

获取指定用户的所有角色。

**请求**:
```protobuf
message GetUserRolesRequest {
  string user_id = 1;  // 用户ID（必填）
}
```

**响应**:
```protobuf
message GetUserRolesResponse {
  repeated Role roles = 1;  // 角色列表
}
```

**需要认证**: ✅

---

##### 5. ListRoles - 列出所有角色

列出系统中的所有角色。

**请求**:
```protobuf
message ListRolesRequest {
  string tenant_id = 1;  // 租户ID过滤
  int32 page = 2;
  int32 page_size = 3;
}
```

**需要认证**: ✅
**需要权限**: `roles.list`

---

## 财务模块

### GL Service

**服务名**: `fi.gl.v1.GlJournalEntryService`
**端口**: 50060

#### 方法列表

##### 1. CreateJournalEntry - 创建会计分录

创建新的会计分录（草稿或立即过账）。

**请求**:
```protobuf
message CreateJournalEntryRequest {
  string company_code = 1;        // 公司代码（必填）
  string document_date = 2;       // 凭证日期（必填）
  string posting_date = 3;        // 过账日期（必填）
  string document_type = 4;       // 凭证类型
  string reference = 5;           // 参考号
  string header_text = 6;         // 抬头文本
  repeated LineItem line_items = 7; // 分录行（必填）
  bool post_immediately = 8;      // 是否立即过账
}

message LineItem {
  string account = 1;             // 科目代码（必填）
  string debit_credit = 2;        // 借贷标识: "D" 或 "C"
  double amount = 3;              // 金额（必填）
  string currency = 4;            // 币种
  string cost_center = 5;         // 成本中心
  string profit_center = 6;       // 利润中心
  string text = 7;                // 行文本
}
```

**响应**:
```protobuf
message JournalEntryResponse {
  string entry_id = 1;            // 分录ID
  string document_number = 2;     // 凭证号
  string status = 3;              // 状态: DRAFT, POSTED, REVERSED
  string message = 4;             // 消息
}
```

**示例**:
```bash
grpcurl -plaintext -d '{
  "company_code": "1000",
  "document_date": "2026-01-19",
  "posting_date": "2026-01-19",
  "document_type": "SA",
  "reference": "INV-2026-001",
  "header_text": "销售收入",
  "line_items": [
    {
      "account": "110000",
      "debit_credit": "D",
      "amount": 10000,
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
}' localhost:50060 fi.gl.v1.GlJournalEntryService/CreateJournalEntry
```

**需要认证**: ✅
**需要权限**: `gl.journal_entry.create`

---

##### 2. PostJournalEntry - 过账分录

将草稿分录过账。

**请求**:
```protobuf
message PostJournalEntryRequest {
  string entry_id = 1;  // 分录ID（必填）
}
```

**需要认证**: ✅
**需要权限**: `gl.journal_entry.post`

---

##### 3. ReverseJournalEntry - 冲销分录

冲销已过账的分录。

**请求**:
```protobuf
message ReverseJournalEntryRequest {
  string entry_id = 1;           // 原分录ID（必填）
  string reversal_date = 2;      // 冲销日期（必填）
  string reversal_reason = 3;    // 冲销原因
}
```

**需要认证**: ✅
**需要权限**: `gl.journal_entry.reverse`

---

##### 4. ListJournalEntries - 查询分录列表

查询会计分录列表。

**请求**:
```protobuf
message ListJournalEntriesRequest {
  string company_code = 1;       // 公司代码
  string from_date = 2;          // 开始日期
  string to_date = 3;            // 结束日期
  string status = 4;             // 状态过滤
  int32 page = 5;                // 页码
  int32 page_size = 6;           // 每页数量
}
```

**响应**:
```protobuf
message ListJournalEntriesResponse {
  repeated JournalEntryDetail entries = 1;
  int32 total_count = 2;
  int32 page = 3;
  int32 page_size = 4;
}
```

**需要认证**: ✅
**需要权限**: `gl.journal_entry.list`

---

##### 5. GetAccountLineItems - 获取科目明细

获取指定科目的明细账。

**请求**:
```protobuf
message GetAccountLineItemsRequest {
  string company_code = 1;       // 公司代码（必填）
  string account = 2;            // 科目代码（必填）
  string from_date = 3;          // 开始日期
  string to_date = 4;            // 结束日期
  string cost_center = 5;        // 成本中心过滤
}
```

**需要认证**: ✅
**需要权限**: `gl.account.view`

---

##### 6. SimulateJournalEntry - 模拟过账

模拟分录过账效果，不实际过账。

**请求**: 同 `CreateJournalEntryRequest`

**响应**:
```protobuf
message SimulationResponse {
  bool is_valid = 1;             // 是否有效
  repeated string errors = 2;    // 错误列表
  repeated string warnings = 3;  // 警告列表
  AccountBalance balance_impact = 4; // 余额影响
}
```

**需要认证**: ✅

---

##### 7. ExecutePeriodEndClose - 执行期末关账

执行会计期间的期末关账。

**请求**:
```protobuf
message ExecutePeriodEndCloseRequest {
  string company_code = 1;       // 公司代码（必填）
  string fiscal_year = 2;        // 会计年度（必填）
  string period = 3;             // 期间（必填）
}
```

**需要认证**: ✅
**需要权限**: `gl.period.close`

---

##### 8. RevaluateForeignCurrency - 外币重估

执行外币科目的重估。

**请求**:
```protobuf
message RevaluateForeignCurrencyRequest {
  string company_code = 1;
  string valuation_date = 2;     // 重估日期（必填）
  repeated string accounts = 3;  // 要重估的科目列表
}
```

**需要认证**: ✅
**需要权限**: `gl.revaluation.execute`

---

### AP Service

**服务名**: `fi.ap.v1.AccountsPayableService`
**端口**: 50061

应付账款服务，管理供应商发票和付款。

#### 主要功能

- 供应商发票管理
- 付款处理
- 供应商对账
- 账龄分析

---

### AR Service

**服务名**: `fi.ar.v1.AccountsReceivableService`
**端口**: 50062

应收账款服务，管理客户发票和收款。

#### 主要功能

- 客户发票管理
- 收款处理
- 客户对账
- 账龄分析
- 催款管理

---

### COA Service

**服务名**: `fi.coa.v1.ChartOfAccountsService`
**端口**: 50065

会计科目表服务，管理科目结构和成本分配。

#### 主要功能

- 科目主数据管理
- 科目层级结构
- 成本中心分配
- 利润中心管理

---

## 通用规范

### 日期格式

所有日期字段使用 ISO 8601 格式：`YYYY-MM-DD`

示例：`2026-01-19`

### 时间戳格式

时间戳使用 RFC 3339 格式：`YYYY-MM-DDTHH:MM:SSZ`

示例：`2026-01-19T12:00:00Z`

### 分页

分页参数：
- `page`: 页码（从1开始）
- `page_size`: 每页数量（默认20，最大100）

分页响应：
- `total_count`: 总记录数
- `page`: 当前页码
- `page_size`: 每页数量

### 币种代码

使用 ISO 4217 标准：
- `CNY` - 人民币
- `USD` - 美元
- `EUR` - 欧元
- `JPY` - 日元

### 租户隔离

所有请求需要指定 `tenant_id`，实现多租户数据隔离。

---

## 错误处理

### gRPC 状态码

| 状态码 | 说明 | 示例 |
|--------|------|------|
| `OK` | 成功 | 请求成功完成 |
| `INVALID_ARGUMENT` | 参数错误 | 必填字段缺失、格式错误 |
| `UNAUTHENTICATED` | 未认证 | Token缺失或无效 |
| `PERMISSION_DENIED` | 权限不足 | 无权限执行操作 |
| `NOT_FOUND` | 资源不存在 | 用户、分录等不存在 |
| `ALREADY_EXISTS` | 资源已存在 | 用户名、邮箱重复 |
| `INTERNAL` | 服务器错误 | 数据库错误、服务异常 |
| `UNAVAILABLE` | 服务不可用 | 服务暂时不可用 |

### 错误响应格式

```json
{
  "code": "INVALID_ARGUMENT",
  "message": "username is required",
  "details": [
    {
      "field": "username",
      "error": "field is required"
    }
  ]
}
```

---

## 示例代码

### Python 客户端

```python
import grpc
from proto import auth_pb2, auth_pb2_grpc

# 创建连接
channel = grpc.insecure_channel('localhost:50051')
stub = auth_pb2_grpc.AuthServiceStub(channel)

# 注册用户
register_request = auth_pb2.RegisterRequest(
    username='john_doe',
    email='john@example.com',
    password='SecurePass123!',
    tenant_id='default'
)
response = stub.Register(register_request)
print(f"User ID: {response.user_id}")

# 登录
login_request = auth_pb2.LoginRequest(
    username='john_doe',
    password='SecurePass123!',
    tenant_id='default'
)
login_response = stub.Login(login_request)
access_token = login_response.access_token

# 使用 Token 调用需要认证的 API
metadata = [('authorization', f'Bearer {access_token}')]
user_response = stub.GetCurrentUser(
    auth_pb2.Empty(),
    metadata=metadata
)
print(f"Current user: {user_response.username}")
```

### Go 客户端

```go
package main

import (
    "context"
    "log"

    "google.golang.org/grpc"
    pb "your-project/proto/auth/v1"
)

func main() {
    // 创建连接
    conn, err := grpc.Dial("localhost:50051", grpc.WithInsecure())
    if err != nil {
        log.Fatal(err)
    }
    defer conn.Close()

    client := pb.NewAuthServiceClient(conn)

    // 注册用户
    registerReq := &pb.RegisterRequest{
        Username: "john_doe",
        Email:    "john@example.com",
        Password: "SecurePass123!",
        TenantId: "default",
    }

    resp, err := client.Register(context.Background(), registerReq)
    if err != nil {
        log.Fatal(err)
    }

    log.Printf("User ID: %s", resp.UserId)
}
```

### Node.js 客户端

```javascript
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');

// 加载 proto 文件
const packageDefinition = protoLoader.loadSync('auth.proto');
const authProto = grpc.loadPackageDefinition(packageDefinition).iam.auth.v1;

// 创建客户端
const client = new authProto.AuthService(
  'localhost:50051',
  grpc.credentials.createInsecure()
);

// 注册用户
client.Register({
  username: 'john_doe',
  email: 'john@example.com',
  password: 'SecurePass123!',
  tenant_id: 'default'
}, (error, response) => {
  if (error) {
    console.error(error);
    return;
  }
  console.log('User ID:', response.user_id);
});
```

### cURL (通过 grpcurl)

```bash
# 注册用户
grpcurl -plaintext -d '{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "SecurePass123!",
  "tenant_id": "default"
}' localhost:50051 iam.auth.v1.AuthService/Register

# 登录
grpcurl -plaintext -d '{
  "username": "john_doe",
  "password": "SecurePass123!",
  "tenant_id": "default"
}' localhost:50051 iam.auth.v1.AuthService/Login

# 使用 Token 调用 API
grpcurl -plaintext \
  -H "authorization: Bearer YOUR_TOKEN" \
  localhost:50051 iam.auth.v1.AuthService/GetCurrentUser
```

---

## 附录

### 快速参考

**列出所有服务**:
```bash
grpcurl -plaintext localhost:50051 list
```

**查看服务方法**:
```bash
grpcurl -plaintext localhost:50051 describe iam.auth.v1.AuthService
```

**查看消息结构**:
```bash
grpcurl -plaintext localhost:50051 describe iam.auth.v1.RegisterRequest
```

### 相关资源

- [gRPC 官方文档](https://grpc.io/docs/)
- [Protocol Buffers 文档](https://protobuf.dev/)
- [grpcurl 工具](https://github.com/fullstorydev/grpcurl)
- [CUBA ERP 项目仓库](https://github.com/your-org/cuba-erp)

---

**文档维护**: CUBA Enterprise Team
**联系方式**: dev@cuba.local
**最后更新**: 2026-01-19
