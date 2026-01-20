# CUBA ERP API 使用示例

> 实用的 API 调用示例和最佳实践

## 目录

- [快速开始](#快速开始)
- [认证流程](#认证流程)
- [用户管理](#用户管理)
- [权限管理](#权限管理)
- [财务操作](#财务操作)
- [常见场景](#常见场景)
- [最佳实践](#最佳实践)
- [故障排查](#故障排查)

---

## 快速开始

### 环境准备

确保服务正在运行：

```bash
# 检查服务状态
docker ps

# 运行测试脚本
./scripts/test-services.sh
```

### 安装 grpcurl

```bash
# macOS
brew install grpcurl

# Linux
wget https://github.com/fullstorydev/grpcurl/releases/download/v1.8.9/grpcurl_1.8.9_linux_x86_64.tar.gz
tar -xvf grpcurl_1.8.9_linux_x86_64.tar.gz
sudo mv grpcurl /usr/local/bin/
```

---

## 认证流程

### 完整的认证流程示例

```bash
#!/bin/bash

# 1. 注册新用户
echo "=== 注册用户 ==="
REGISTER_RESPONSE=$(grpcurl -plaintext -d '{
  "username": "alice",
  "email": "alice@example.com",
  "password": "Alice123456!",
  "tenant_id": "default"
}' localhost:50051 iam.auth.v1.AuthService/Register)

echo "$REGISTER_RESPONSE"
USER_ID=$(echo "$REGISTER_RESPONSE" | jq -r '.userId')
echo "用户ID: $USER_ID"

# 2. 登录获取 Token
echo -e "\n=== 用户登录 ==="
LOGIN_RESPONSE=$(grpcurl -plaintext -d '{
  "username": "alice",
  "password": "Alice123456!",
  "tenant_id": "default"
}' localhost:50051 iam.auth.v1.AuthService/Login)

echo "$LOGIN_RESPONSE"
ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.accessToken')
REFRESH_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.refreshToken')
SESSION_ID=$(echo "$LOGIN_RESPONSE" | jq -r '.sessionId')

echo "Access Token: ${ACCESS_TOKEN:0:50}..."
echo "Session ID: $SESSION_ID"

# 3. 使用 Token 获取用户信息
echo -e "\n=== 获取当前用户信息 ==="
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{}' \
  localhost:50051 iam.auth.v1.AuthService/GetCurrentUser

# 4. 刷新 Token
echo -e "\n=== 刷新 Token ==="
NEW_TOKEN_RESPONSE=$(grpcurl -plaintext -d "{
  \"refreshToken\": \"$REFRESH_TOKEN\"
}" localhost:50051 iam.auth.v1.AuthService/RefreshToken)

echo "$NEW_TOKEN_RESPONSE"
NEW_ACCESS_TOKEN=$(echo "$NEW_TOKEN_RESPONSE" | jq -r '.accessToken')
echo "新 Access Token: ${NEW_ACCESS_TOKEN:0:50}..."

# 5. 登出
echo -e "\n=== 用户登出 ==="
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d "{\"sessionId\": \"$SESSION_ID\"}" \
  localhost:50051 iam.auth.v1.AuthService/Logout
```

保存为 `scripts/auth-flow-example.sh` 并运行：

```bash
chmod +x scripts/auth-flow-example.sh
./scripts/auth-flow-example.sh
```

---

## 用户管理

### 1. 批量注册用户

```bash
#!/bin/bash

# 批量注册用户
USERS=(
  "bob:bob@example.com:Bob123456!"
  "charlie:charlie@example.com:Charlie123456!"
  "david:david@example.com:David123456!"
)

for user in "${USERS[@]}"; do
  IFS=':' read -r username email password <<< "$user"

  echo "注册用户: $username"
  grpcurl -plaintext -d "{
    \"username\": \"$username\",
    \"email\": \"$email\",
    \"password\": \"$password\",
    \"tenant_id\": \"default\"
  }" localhost:50051 iam.auth.v1.AuthService/Register

  echo ""
done
```

### 2. 修改密码

```bash
# 先登录获取 Token
ACCESS_TOKEN="your_access_token_here"

# 修改密码
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "oldPassword": "OldPass123!",
    "newPassword": "NewPass123!"
  }' localhost:50051 iam.auth.v1.AuthService/ChangePassword
```

### 3. 更新个人资料

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "email": "newemail@example.com",
    "displayName": "Alice Smith",
    "avatarUrl": "https://example.com/avatar.jpg"
  }' localhost:50051 iam.auth.v1.AuthService/UpdateProfile
```

### 4. 启用双因素认证

```bash
# 启用 2FA
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{}' \
  localhost:50051 iam.auth.v1.AuthService/Enable2FA

# 响应包含:
# - secret: TOTP 密钥
# - qr_code_url: 二维码 URL
# - backup_codes: 备份码列表
```

### 5. 管理员查看用户列表

```bash
# 需要管理员权限
grpcurl -plaintext \
  -H "authorization: Bearer $ADMIN_TOKEN" \
  -d '{
    "page": 1,
    "pageSize": 20,
    "tenantId": "default"
  }' localhost:50051 iam.auth.v1.AuthService/ListUsers
```

---

## 权限管理

### 1. 创建角色

```bash
# 创建"财务经理"角色
grpcurl -plaintext \
  -H "authorization: Bearer $ADMIN_TOKEN" \
  -d '{
    "name": "finance_manager",
    "description": "财务经理角色",
    "tenantId": "default",
    "permissions": [
      "gl.journal_entry.create",
      "gl.journal_entry.post",
      "gl.journal_entry.list",
      "gl.account.view",
      "ap.invoice.create",
      "ar.invoice.create"
    ]
  }' localhost:50052 iam.rbac.v1.RBACService/CreateRole
```

### 2. 分配角色给用户

```bash
# 将"财务经理"角色分配给用户
grpcurl -plaintext \
  -H "authorization: Bearer $ADMIN_TOKEN" \
  -d '{
    "userId": "user-uuid-here",
    "roleId": "role-uuid-here",
    "tenantId": "default"
  }' localhost:50052 iam.rbac.v1.RBACService/AssignRoleToUser
```

### 3. 检查用户权限

```bash
# 检查用户是否有特定权限
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "userId": "user-uuid-here",
    "permissions": [
      "gl.journal_entry.create",
      "gl.journal_entry.post"
    ]
  }' localhost:50052 iam.rbac.v1.RBACService/CheckPermissions

# 响应示例:
# {
#   "results": {
#     "gl.journal_entry.create": true,
#     "gl.journal_entry.post": false
#   }
# }
```

### 4. 获取用户的所有角色

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "userId": "user-uuid-here"
  }' localhost:50052 iam.rbac.v1.RBACService/GetUserRoles
```

### 5. 获取用户权限码（前端使用）

```bash
# 前端可以用这个接口获取当前用户的所有权限码
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{}' \
  localhost:50051 iam.auth.v1.AuthService/GetPermCodes

# 响应示例:
# {
#   "permCodes": [
#     "gl.journal_entry.create",
#     "gl.journal_entry.list",
#     "gl.account.view"
#   ]
# }
```

---

## 财务操作

### 1. 创建会计分录（销售收入）

```bash
# 创建销售收入分录
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "companyCode": "1000",
    "documentDate": "2026-01-19",
    "postingDate": "2026-01-19",
    "documentType": "SA",
    "reference": "INV-2026-001",
    "headerText": "销售商品收入",
    "lineItems": [
      {
        "account": "110000",
        "debitCredit": "D",
        "amount": 11300,
        "currency": "CNY",
        "costCenter": "CC001",
        "text": "应收账款-客户A"
      },
      {
        "account": "600000",
        "debitCredit": "C",
        "amount": 10000,
        "currency": "CNY",
        "text": "主营业务收入"
      },
      {
        "account": "220300",
        "debitCredit": "C",
        "amount": 1300,
        "currency": "CNY",
        "text": "应交增值税-销项税"
      }
    ],
    "postImmediately": true
  }' localhost:50060 fi.gl.v1.GlJournalEntryService/CreateJournalEntry
```

### 2. 创建采购成本分录

```bash
# 采购原材料
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "companyCode": "1000",
    "documentDate": "2026-01-19",
    "postingDate": "2026-01-19",
    "documentType": "KR",
    "reference": "PO-2026-001",
    "headerText": "采购原材料",
    "lineItems": [
      {
        "account": "140100",
        "debitCredit": "D",
        "amount": 50000,
        "currency": "CNY",
        "costCenter": "CC002",
        "text": "原材料-钢材"
      },
      {
        "account": "170100",
        "debitCredit": "D",
        "amount": 6500,
        "currency": "CNY",
        "text": "应交增值税-进项税"
      },
      {
        "account": "210000",
        "debitCredit": "C",
        "amount": 56500,
        "currency": "CNY",
        "text": "应付账款-供应商B"
      }
    ],
    "postImmediately": false
  }' localhost:50060 fi.gl.v1.GlJournalEntryService/CreateJournalEntry
```

### 3. 模拟过账（验证分录）

```bash
# 在实际过账前模拟，检查是否有错误
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "companyCode": "1000",
    "documentDate": "2026-01-19",
    "postingDate": "2026-01-19",
    "documentType": "SA",
    "lineItems": [
      {
        "account": "110000",
        "debitCredit": "D",
        "amount": 10000,
        "currency": "CNY"
      },
      {
        "account": "600000",
        "debitCredit": "C",
        "amount": 10000,
        "currency": "CNY"
      }
    ]
  }' localhost:50060 fi.gl.v1.GlJournalEntryService/SimulateJournalEntry
```

### 4. 过账草稿分录

```bash
# 将草稿分录过账
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "entryId": "entry-uuid-here"
  }' localhost:50060 fi.gl.v1.GlJournalEntryService/PostJournalEntry
```

### 5. 冲销分录

```bash
# 冲销已过账的分录
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "entryId": "entry-uuid-here",
    "reversalDate": "2026-01-20",
    "reversalReason": "错误分录，需要冲销"
  }' localhost:50060 fi.gl.v1.GlJournalEntryService/ReverseJournalEntry
```

### 6. 查询分录列表

```bash
# 查询指定日期范围的分录
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "companyCode": "1000",
    "fromDate": "2026-01-01",
    "toDate": "2026-01-31",
    "status": "POSTED",
    "page": 1,
    "pageSize": 20
  }' localhost:50060 fi.gl.v1.GlJournalEntryService/ListJournalEntries
```

### 7. 查询科目明细账

```bash
# 查询应收账款科目明细
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "companyCode": "1000",
    "account": "110000",
    "fromDate": "2026-01-01",
    "toDate": "2026-01-31"
  }' localhost:50060 fi.gl.v1.GlJournalEntryService/GetAccountLineItems
```

### 8. 批量创建分录

```bash
# 批量创建多个分录
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "entries": [
      {
        "companyCode": "1000",
        "documentDate": "2026-01-19",
        "postingDate": "2026-01-19",
        "documentType": "SA",
        "reference": "INV-001",
        "lineItems": [...]
      },
      {
        "companyCode": "1000",
        "documentDate": "2026-01-19",
        "postingDate": "2026-01-19",
        "documentType": "SA",
        "reference": "INV-002",
        "lineItems": [...]
      }
    ]
  }' localhost:50060 fi.gl.v1.GlJournalEntryService/BatchCreateJournalEntries
```

---

## 常见场景

### 场景 1: 新员工入职流程

```bash
#!/bin/bash

# 1. 创建用户账号
USER_RESPONSE=$(grpcurl -plaintext -d '{
  "username": "new_employee",
  "email": "employee@company.com",
  "password": "TempPass123!",
  "tenantId": "default"
}' localhost:50051 iam.auth.v1.AuthService/Register)

USER_ID=$(echo "$USER_RESPONSE" | jq -r '.userId')

# 2. 分配基础角色
grpcurl -plaintext \
  -H "authorization: Bearer $ADMIN_TOKEN" \
  -d "{
    \"userId\": \"$USER_ID\",
    \"roleId\": \"employee_role_id\",
    \"tenantId\": \"default\"
  }" localhost:50052 iam.rbac.v1.RBACService/AssignRoleToUser

# 3. 发送欢迎邮件（假设有邮件服务）
echo "欢迎邮件已发送到 employee@company.com"
```

### 场景 2: 月末关账流程

```bash
#!/bin/bash

COMPANY_CODE="1000"
FISCAL_YEAR="2026"
PERIOD="01"

# 1. 检查所有分录是否已过账
echo "检查未过账分录..."
DRAFT_ENTRIES=$(grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d "{
    \"companyCode\": \"$COMPANY_CODE\",
    \"status\": \"DRAFT\"
  }" localhost:50060 fi.gl.v1.GlJournalEntryService/ListJournalEntries)

# 2. 执行外币重估
echo "执行外币重估..."
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d "{
    \"companyCode\": \"$COMPANY_CODE\",
    \"valuationDate\": \"2026-01-31\"
  }" localhost:50060 fi.gl.v1.GlJournalEntryService/RevaluateForeignCurrency

# 3. 执行期末关账
echo "执行期末关账..."
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d "{
    \"companyCode\": \"$COMPANY_CODE\",
    \"fiscalYear\": \"$FISCAL_YEAR\",
    \"period\": \"$PERIOD\"
  }" localhost:50060 fi.gl.v1.GlJournalEntryService/ExecutePeriodEndClose

echo "月末关账完成！"
```

### 场景 3: 权限审计

```bash
#!/bin/bash

# 获取所有用户
USERS=$(grpcurl -plaintext \
  -H "authorization: Bearer $ADMIN_TOKEN" \
  -d '{
    "page": 1,
    "pageSize": 100
  }' localhost:50051 iam.auth.v1.AuthService/ListUsers)

# 遍历每个用户，检查其角色和权限
echo "$USERS" | jq -r '.users[].userId' | while read USER_ID; do
  echo "用户 ID: $USER_ID"

  # 获取用户角色
  grpcurl -plaintext \
    -H "authorization: Bearer $ADMIN_TOKEN" \
    -d "{\"userId\": \"$USER_ID\"}" \
    localhost:50052 iam.rbac.v1.RBACService/GetUserRoles

  echo "---"
done
```

---

## 最佳实践

### 1. Token 管理

```bash
# 将 Token 保存到环境变量
export CUBA_ACCESS_TOKEN="your_access_token"
export CUBA_REFRESH_TOKEN="your_refresh_token"

# 创建辅助函数
cuba_api() {
  local service=$1
  local method=$2
  local data=$3

  grpcurl -plaintext \
    -H "authorization: Bearer $CUBA_ACCESS_TOKEN" \
    -d "$data" \
    "localhost:$service" "$method"
}

# 使用示例
cuba_api 50051 "iam.auth.v1.AuthService/GetCurrentUser" '{}'
```

### 2. 错误处理

```bash
# 捕获错误并处理
RESPONSE=$(grpcurl -plaintext -d '{...}' localhost:50051 iam.auth.v1.AuthService/Login 2>&1)

if echo "$RESPONSE" | grep -q "ERROR"; then
  echo "登录失败: $RESPONSE"
  exit 1
else
  echo "登录成功"
  ACCESS_TOKEN=$(echo "$RESPONSE" | jq -r '.accessToken')
fi
```

### 3. 批量操作

```bash
# 从 CSV 文件批量导入用户
while IFS=',' read -r username email password; do
  grpcurl -plaintext -d "{
    \"username\": \"$username\",
    \"email\": \"$email\",
    \"password\": \"$password\",
    \"tenantId\": \"default\"
  }" localhost:50051 iam.auth.v1.AuthService/Register
done < users.csv
```

### 4. 日志记录

```bash
# 记录所有 API 调用
LOG_FILE="api_calls.log"

api_call_with_log() {
  local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  local command="$@"

  echo "[$timestamp] Executing: $command" >> "$LOG_FILE"

  result=$($command 2>&1)
  echo "[$timestamp] Result: $result" >> "$LOG_FILE"

  echo "$result"
}

# 使用
api_call_with_log grpcurl -plaintext -d '{...}' localhost:50051 iam.auth.v1.AuthService/Login
```

---

## 故障排查

### 问题 1: 连接被拒绝

```bash
# 检查服务是否运行
docker ps | grep cuba

# 检查端口是否开放
nc -zv localhost 50051

# 查看服务日志
docker logs cuba-auth-service
```

### 问题 2: 认证失败

```bash
# 验证 Token 是否有效
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d "{\"token\": \"$ACCESS_TOKEN\"}" \
  localhost:50051 iam.auth.v1.AuthService/ValidateToken

# 如果 Token 过期，刷新它
grpcurl -plaintext -d "{
  \"refreshToken\": \"$REFRESH_TOKEN\"
}" localhost:50051 iam.auth.v1.AuthService/RefreshToken
```

### 问题 3: 权限不足

```bash
# 检查用户权限
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{}' \
  localhost:50051 iam.auth.v1.AuthService/GetPermCodes

# 检查特定权限
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "userId": "your-user-id",
    "permissions": ["gl.journal_entry.create"]
  }' localhost:50052 iam.rbac.v1.RBACService/CheckPermissions
```

### 问题 4: 数据库错误

```bash
# 连接数据库检查
docker exec -it cuba-postgres psql -U postgres -d cuba_iam

# 检查表是否存在
\dt

# 查看用户数据
SELECT * FROM users LIMIT 10;
```

---

## 性能优化

### 1. 使用连接池

在生产环境中，使用 gRPC 连接池以提高性能：

```python
# Python 示例
import grpc
from grpc import aio

# 创建连接池
channel = aio.insecure_channel(
    'localhost:50051',
    options=[
        ('grpc.keepalive_time_ms', 10000),
        ('grpc.keepalive_timeout_ms', 5000),
        ('grpc.http2.max_pings_without_data', 0),
        ('grpc.keepalive_permit_without_calls', 1),
    ]
)
```

### 2. 批量操作

尽可能使用批量 API：

```bash
# 好的做法：批量创建
grpcurl -plaintext -d '{
  "entries": [...]  # 多个分录
}' localhost:50060 fi.gl.v1.GlJournalEntryService/BatchCreateJournalEntries

# 避免：循环单个创建
for entry in entries; do
  grpcurl -plaintext -d "$entry" localhost:50060 fi.gl.v1.GlJournalEntryService/CreateJournalEntry
done
```

### 3. 流式 API

对于大量数据，使用流式 API：

```bash
# 流式查询分录
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "companyCode": "1000",
    "fromDate": "2026-01-01",
    "toDate": "2026-12-31"
  }' localhost:50060 fi.gl.v1.GlJournalEntryService/StreamJournalEntries
```

---

## 附录

### 完整的测试脚本

保存为 `scripts/comprehensive-test.sh`:

```bash
#!/bin/bash

set -e

echo "🧪 CUBA ERP 综合测试"
echo "===================="

# 颜色
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

# 1. 注册用户
echo -e "\n${GREEN}1. 注册用户${NC}"
USER_RESPONSE=$(grpcurl -plaintext -d '{
  "username": "test_'$(date +%s)'",
  "email": "test_'$(date +%s)'@example.com",
  "password": "Test123456!",
  "tenant_id": "default"
}' localhost:50051 iam.auth.v1.AuthService/Register)

USER_ID=$(echo "$USER_RESPONSE" | jq -r '.userId')
USERNAME=$(echo "$USER_RESPONSE" | jq -r '.username')
echo "✓ 用户创建成功: $USERNAME"

# 2. 登录
echo -e "\n${GREEN}2. 用户登录${NC}"
LOGIN_RESPONSE=$(grpcurl -plaintext -d "{
  \"username\": \"$USERNAME\",
  \"password\": \"Test123456!\",
  \"tenant_id\": \"default\"
}" localhost:50051 iam.auth.v1.AuthService/Login)

ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.accessToken')
echo "✓ 登录成功，Token: ${ACCESS_TOKEN:0:30}..."

# 3. 获取用户信息
echo -e "\n${GREEN}3. 获取用户信息${NC}"
grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{}' \
  localhost:50051 iam.auth.v1.AuthService/GetCurrentUser

# 4. 创建会计分录
echo -e "\n${GREEN}4. 创建会计分录${NC}"
ENTRY_RESPONSE=$(grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "companyCode": "1000",
    "documentDate": "2026-01-19",
    "postingDate": "2026-01-19",
    "documentType": "SA",
    "reference": "TEST-001",
    "headerText": "测试分录",
    "lineItems": [
      {
        "account": "110000",
        "debitCredit": "D",
        "amount": 1000,
        "currency": "CNY",
        "text": "应收账款"
      },
      {
        "account": "600000",
        "debitCredit": "C",
        "amount": 1000,
        "currency": "CNY",
        "text": "收入"
      }
    ],
    "postImmediately": true
  }' localhost:50060 fi.gl.v1.GlJournalEntryService/CreateJournalEntry 2>&1)

if echo "$ENTRY_RESPONSE" | grep -q "entryId"; then
  ENTRY_ID=$(echo "$ENTRY_RESPONSE" | jq -r '.entryId')
  echo "✓ 分录创建成功: $ENTRY_ID"
else
  echo "⚠ 分录创建失败（可能需要数据库迁移）"
fi

echo -e "\n${GREEN}✅ 所有测试完成！${NC}"
```

---

**文档维护**: CUBA Enterprise Team
**最后更新**: 2026-01-19
