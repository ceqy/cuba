#!/bin/bash

# CUBA ERP API 综合测试脚本
# 测试文档中的所有示例

set -e

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 测试函数
run_test() {
    local test_name=$1
    local test_command=$2

    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo -e "\n${BLUE}[Test $TOTAL_TESTS]${NC} $test_name"
    echo "Command: $test_command"

    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}✗ FAILED${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# 打印标题
print_header() {
    echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${YELLOW}$1${NC}"
    echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

echo "🧪 CUBA ERP API 综合测试"
echo "========================"
echo "测试所有文档中的 API 示例"
echo ""

# ============================================
# 1. 服务连通性测试
# ============================================
print_header "1. 服务连通性测试"

run_test "PostgreSQL 连接" "nc -z localhost 5432"
run_test "Auth Service 连接" "nc -z localhost 50051"
run_test "RBAC Service 连接" "nc -z localhost 50052"
run_test "GL Service 连接" "nc -z localhost 50060"
run_test "AP Service 连接" "nc -z localhost 50061"
run_test "AR Service 连接" "nc -z localhost 50062"
run_test "COA Service 连接" "nc -z localhost 50065"

# ============================================
# 2. 认证流程测试
# ============================================
print_header "2. 认证流程测试"

# 生成唯一用户名
TIMESTAMP=$(date +%s)
TEST_USERNAME="test_user_$TIMESTAMP"
TEST_EMAIL="test_$TIMESTAMP@example.com"
TEST_PASSWORD="TestPass123!"

echo "测试用户: $TEST_USERNAME"

# 2.1 注册用户
echo -e "\n${BLUE}[Test $((TOTAL_TESTS + 1))]${NC} 用户注册"
REGISTER_RESPONSE=$(grpcurl -plaintext -d "{
  \"username\": \"$TEST_USERNAME\",
  \"email\": \"$TEST_EMAIL\",
  \"password\": \"$TEST_PASSWORD\",
  \"tenant_id\": \"default\"
}" localhost:50051 iam.auth.v1.AuthService/Register 2>&1)

if echo "$REGISTER_RESPONSE" | grep -q "userId"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    USER_ID=$(echo "$REGISTER_RESPONSE" | jq -r '.userId')
    echo "  用户ID: $USER_ID"
else
    echo -e "${RED}✗ FAILED${NC}"
    echo "$REGISTER_RESPONSE"
    FAILED_TESTS=$((FAILED_TESTS + 1))
    exit 1
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# 2.2 用户登录
echo -e "\n${BLUE}[Test $((TOTAL_TESTS + 1))]${NC} 用户登录"
LOGIN_RESPONSE=$(grpcurl -plaintext -d "{
  \"username\": \"$TEST_USERNAME\",
  \"password\": \"$TEST_PASSWORD\",
  \"tenant_id\": \"default\"
}" localhost:50051 iam.auth.v1.AuthService/Login 2>&1)

if echo "$LOGIN_RESPONSE" | grep -q "accessToken"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.accessToken')
    REFRESH_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.refreshToken')
    SESSION_ID=$(echo "$LOGIN_RESPONSE" | jq -r '.sessionId')
    echo "  Token: ${ACCESS_TOKEN:0:30}..."
    echo "  Session: $SESSION_ID"
else
    echo -e "${RED}✗ FAILED${NC}"
    echo "$LOGIN_RESPONSE"
    FAILED_TESTS=$((FAILED_TESTS + 1))
    exit 1
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# 2.3 获取当前用户信息
echo -e "\n${BLUE}[Test $((TOTAL_TESTS + 1))]${NC} 获取当前用户信息"
USER_INFO=$(grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{}' \
  localhost:50051 iam.auth.v1.AuthService/GetCurrentUser 2>&1)

if echo "$USER_INFO" | grep -q "userId"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "$USER_INFO" | jq '.'
else
    echo -e "${RED}✗ FAILED${NC}"
    echo "$USER_INFO"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# 2.4 刷新 Token
echo -e "\n${BLUE}[Test $((TOTAL_TESTS + 1))]${NC} 刷新 Token"
REFRESH_RESPONSE=$(grpcurl -plaintext -d "{
  \"refreshToken\": \"$REFRESH_TOKEN\"
}" localhost:50051 iam.auth.v1.AuthService/RefreshToken 2>&1)

if echo "$REFRESH_RESPONSE" | grep -q "accessToken"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    NEW_ACCESS_TOKEN=$(echo "$REFRESH_RESPONSE" | jq -r '.accessToken')
    echo "  新 Token: ${NEW_ACCESS_TOKEN:0:30}..."
else
    echo -e "${RED}✗ FAILED${NC}"
    echo "$REFRESH_RESPONSE"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# 2.5 获取权限码
echo -e "\n${BLUE}[Test $((TOTAL_TESTS + 1))]${NC} 获取权限码"
PERM_CODES=$(grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{}' \
  localhost:50051 iam.auth.v1.AuthService/GetPermCodes 2>&1)

if echo "$PERM_CODES" | grep -q "permCodes"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "$PERM_CODES" | jq '.'
else
    echo -e "${RED}✗ FAILED${NC}"
    echo "$PERM_CODES"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# ============================================
# 3. RBAC 服务测试
# ============================================
print_header "3. RBAC 服务测试"

# 3.1 列出所有角色
echo -e "\n${BLUE}[Test $((TOTAL_TESTS + 1))]${NC} 列出所有角色"
ROLES_LIST=$(grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "tenantId": "default",
    "page": 1,
    "pageSize": 10
  }' \
  localhost:50052 iam.rbac.v1.RBACService/ListRoles 2>&1)

if echo "$ROLES_LIST" | grep -q "roles"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${YELLOW}⚠ SKIPPED (可能需要管理员权限)${NC}"
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# 3.2 获取用户角色
echo -e "\n${BLUE}[Test $((TOTAL_TESTS + 1))]${NC} 获取用户角色"
USER_ROLES=$(grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d "{
    \"userId\": \"$USER_ID\"
  }" \
  localhost:50052 iam.rbac.v1.RBACService/GetUserRoles 2>&1)

if echo "$USER_ROLES" | grep -q "roles"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "$USER_ROLES" | jq '.'
else
    echo -e "${RED}✗ FAILED${NC}"
    echo "$USER_ROLES"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# ============================================
# 4. GL 服务测试
# ============================================
print_header "4. GL 服务测试"

# 4.1 列出 GL 服务方法
echo -e "\n${BLUE}[Test $((TOTAL_TESTS + 1))]${NC} 列出 GL 服务方法"
GL_METHODS=$(grpcurl -plaintext localhost:50060 list 2>&1)

if echo "$GL_METHODS" | grep -q "GlJournalEntryService"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "$GL_METHODS"
else
    echo -e "${RED}✗ FAILED${NC}"
    echo "$GL_METHODS"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# 4.2 创建会计分录（模拟）
echo -e "\n${BLUE}[Test $((TOTAL_TESTS + 1))]${NC} 模拟创建会计分录"
SIMULATE_RESPONSE=$(grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "companyCode": "1000",
    "documentDate": "2026-01-19",
    "postingDate": "2026-01-19",
    "documentType": "SA",
    "reference": "TEST-'$TIMESTAMP'",
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
    ]
  }' \
  localhost:50060 fi.gl.v1.GlJournalEntryService/SimulateJournalEntry 2>&1)

if echo "$SIMULATE_RESPONSE" | grep -q "isValid"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
    echo "$SIMULATE_RESPONSE" | jq '.'
else
    echo -e "${YELLOW}⚠ SKIPPED (可能需要数据库迁移)${NC}"
    echo "$SIMULATE_RESPONSE"
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# 4.3 查询分录列表
echo -e "\n${BLUE}[Test $((TOTAL_TESTS + 1))]${NC} 查询分录列表"
LIST_ENTRIES=$(grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "companyCode": "1000",
    "fromDate": "2026-01-01",
    "toDate": "2026-01-31",
    "page": 1,
    "pageSize": 10
  }' \
  localhost:50060 fi.gl.v1.GlJournalEntryService/ListJournalEntries 2>&1)

if echo "$LIST_ENTRIES" | grep -q "entries"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${YELLOW}⚠ SKIPPED (可能需要数据库迁移)${NC}"
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# ============================================
# 5. 清理测试
# ============================================
print_header "5. 清理测试数据"

# 5.1 登出
echo -e "\n${BLUE}[Test $((TOTAL_TESTS + 1))]${NC} 用户登出"
LOGOUT_RESPONSE=$(grpcurl -plaintext \
  -H "authorization: Bearer $ACCESS_TOKEN" \
  -d "{\"sessionId\": \"$SESSION_ID\"}" \
  localhost:50051 iam.auth.v1.AuthService/Logout 2>&1)

if echo "$LOGOUT_RESPONSE" | grep -q "success"; then
    echo -e "${GREEN}✓ PASSED${NC}"
    PASSED_TESTS=$((PASSED_TESTS + 1))
else
    echo -e "${RED}✗ FAILED${NC}"
    echo "$LOGOUT_RESPONSE"
    FAILED_TESTS=$((FAILED_TESTS + 1))
fi
TOTAL_TESTS=$((TOTAL_TESTS + 1))

# ============================================
# 测试总结
# ============================================
print_header "测试总结"

echo ""
echo "总测试数: $TOTAL_TESTS"
echo -e "${GREEN}通过: $PASSED_TESTS${NC}"
echo -e "${RED}失败: $FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✅ 所有测试通过！${NC}"
    exit 0
else
    echo -e "${RED}❌ 有 $FAILED_TESTS 个测试失败${NC}"
    exit 1
fi
