#!/bin/bash

# CUBA ERP 服务测试脚本
# 测试所有运行中的微服务

set -e

echo "🧪 CUBA ERP 服务测试"
echo "===================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 测试函数
test_service() {
    local service_name=$1
    local port=$2
    local description=$3

    echo -n "Testing $description ($service_name:$port)... "
    if nc -z localhost $port 2>/dev/null; then
        echo -e "${GREEN}✓${NC}"
        return 0
    else
        echo -e "${RED}✗${NC}"
        return 1
    fi
}

echo "📡 测试服务连通性"
echo "-------------------"
test_service "postgres" 5432 "PostgreSQL 数据库"
test_service "auth-service" 50051 "Auth Service (认证)"
test_service "rbac-service" 50052 "RBAC Service (权限)"
test_service "gl-service" 50060 "GL Service (总账)"
test_service "ap-service" 50061 "AP Service (应付)"
test_service "ar-service" 50062 "AR Service (应收)"
test_service "coa-service" 50065 "COA Service (科目表)"
echo ""

echo "🔐 测试 Auth Service API"
echo "------------------------"

# 1. 注册用户
echo -n "1. 注册新用户... "
REGISTER_RESULT=$(grpcurl -plaintext -d '{
  "username": "demo_'$(date +%s)'",
  "email": "demo_'$(date +%s)'@example.com",
  "password": "Demo123456!",
  "tenant_id": "default"
}' localhost:50051 iam.auth.v1.AuthService/Register 2>&1)

if echo "$REGISTER_RESULT" | grep -q "userId"; then
    echo -e "${GREEN}✓${NC}"
    USER_ID=$(echo "$REGISTER_RESULT" | grep -o '"userId": "[^"]*"' | cut -d'"' -f4)
    USERNAME=$(echo "$REGISTER_RESULT" | grep -o '"username": "[^"]*"' | cut -d'"' -f4)
    echo "   用户ID: $USER_ID"
    echo "   用户名: $USERNAME"
else
    echo -e "${RED}✗${NC}"
    echo "$REGISTER_RESULT"
fi
echo ""

# 2. 登录
echo -n "2. 用户登录... "
LOGIN_RESULT=$(grpcurl -plaintext -d '{
  "username": "'$USERNAME'",
  "password": "Demo123456!",
  "tenant_id": "default"
}' localhost:50051 iam.auth.v1.AuthService/Login 2>&1)

if echo "$LOGIN_RESULT" | grep -q "accessToken"; then
    echo -e "${GREEN}✓${NC}"
    ACCESS_TOKEN=$(echo "$LOGIN_RESULT" | grep -o '"accessToken": "[^"]*"' | cut -d'"' -f4)
    echo "   Token: ${ACCESS_TOKEN:0:50}..."
else
    echo -e "${RED}✗${NC}"
    echo "$LOGIN_RESULT"
fi
echo ""

echo "📊 测试 GL Service API"
echo "----------------------"

# 列出可用的服务
echo "可用的 gRPC 方法:"
grpcurl -plaintext localhost:50060 describe fi.gl.v1.GlJournalEntryService | grep "rpc " | head -5
echo "   ... (共 20+ 个方法)"
echo ""

echo "📋 测试 RBAC Service API"
echo "------------------------"
echo "可用的 gRPC 方法:"
grpcurl -plaintext localhost:50052 describe iam.rbac.v1.RBACService | grep "rpc " | head -5
echo ""

echo "✅ 测试完成！"
echo ""
echo "💡 提示："
echo "   - 使用 'docker ps' 查看所有运行的服务"
echo "   - 使用 'docker logs <service-name> -f' 查看服务日志"
echo "   - 使用 'grpcurl -plaintext localhost:<port> list' 列出服务方法"
echo "   - 使用 'docker exec -it cuba-postgres psql -U postgres' 连接数据库"
echo ""
