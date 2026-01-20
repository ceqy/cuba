#!/bin/bash

# CUBA ERP 测试数据初始化脚本
# 用于在 Swagger UI 中测试 API

set -e

echo "🚀 开始初始化测试数据..."

# API Gateway 地址
API_URL="http://localhost:8080"

# 颜色输出
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 1. 注册测试用户
echo -e "${BLUE}📝 步骤 1: 注册测试用户${NC}"
REGISTER_RESPONSE=$(curl -s -X POST ${API_URL}/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "demo_user",
    "password": "Demo123456",
    "email": "demo@cuba.local",
    "tenant_id": "default"
  }')

echo "$REGISTER_RESPONSE" | jq .
USER_ID=$(echo "$REGISTER_RESPONSE" | jq -r '.user_id')
echo -e "${GREEN}✅ 用户创建成功: $USER_ID${NC}"

# 2. 登录获取 Token
echo -e "\n${BLUE}📝 步骤 2: 登录获取 Token${NC}"
LOGIN_RESPONSE=$(curl -s -X POST ${API_URL}/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "demo_user",
    "password": "Demo123456",
    "tenant_id": "default"
  }')

echo "$LOGIN_RESPONSE" | jq .
TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token')
echo -e "${GREEN}✅ 登录成功，Token: ${TOKEN:0:50}...${NC}"

# 3. 创建角色
echo -e "\n${BLUE}📝 步骤 3: 创建测试角色${NC}"
ROLE_RESPONSE=$(curl -s -X POST ${API_URL}/api/v1/rbac/roles \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "name": "finance_manager",
    "description": "财务经理角色",
    "tenant_id": "default",
    "permissions": [
      "gl.journal_entry.create",
      "gl.journal_entry.post",
      "gl.journal_entry.view",
      "gl.journal_entry.list"
    ]
  }')

echo "$ROLE_RESPONSE" | jq .
ROLE_ID=$(echo "$ROLE_RESPONSE" | jq -r '.role_id // empty')

if [ -n "$ROLE_ID" ]; then
  echo -e "${GREEN}✅ 角色创建成功: $ROLE_ID${NC}"
else
  echo -e "${RED}⚠️  角色创建失败或已存在${NC}"
fi

# 4. 创建会计分录
echo -e "\n${BLUE}📝 步骤 4: 创建测试会计分录${NC}"
JE_RESPONSE=$(curl -s -X POST ${API_URL}/api/v1/finance/gl/journal-entries \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "company_code": "1000",
    "document_date": "2026-01-20",
    "posting_date": "2026-01-20",
    "document_type": "SA",
    "reference": "TEST-001",
    "header_text": "测试销售收入",
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
      },
      {
        "account": "220300",
        "debit_credit": "C",
        "amount": 1300,
        "currency": "CNY",
        "text": "销项税"
      }
    ],
    "post_immediately": false
  }')

echo "$JE_RESPONSE" | jq .
ENTRY_ID=$(echo "$JE_RESPONSE" | jq -r '.entry_id // empty')

if [ -n "$ENTRY_ID" ]; then
  echo -e "${GREEN}✅ 会计分录创建成功: $ENTRY_ID${NC}"
else
  echo -e "${RED}⚠️  会计分录创建失败${NC}"
fi

# 5. 创建第二个会计分录
echo -e "\n${BLUE}📝 步骤 5: 创建第二个测试会计分录${NC}"
JE_RESPONSE2=$(curl -s -X POST ${API_URL}/api/v1/finance/gl/journal-entries \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "company_code": "1000",
    "document_date": "2026-01-20",
    "posting_date": "2026-01-20",
    "document_type": "KR",
    "reference": "TEST-002",
    "header_text": "测试采购成本",
    "line_items": [
      {
        "account": "500000",
        "debit_credit": "D",
        "amount": 5000,
        "currency": "CNY",
        "text": "原材料采购"
      },
      {
        "account": "210000",
        "debit_credit": "C",
        "amount": 5000,
        "currency": "CNY",
        "text": "应付账款"
      }
    ],
    "post_immediately": true
  }')

echo "$JE_RESPONSE2" | jq .
ENTRY_ID2=$(echo "$JE_RESPONSE2" | jq -r '.entry_id // empty')

if [ -n "$ENTRY_ID2" ]; then
  echo -e "${GREEN}✅ 会计分录创建成功并已过账: $ENTRY_ID2${NC}"
else
  echo -e "${RED}⚠️  会计分录创建失败${NC}"
fi

# 6. 输出测试信息
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ 测试数据初始化完成!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "\n${BLUE}📋 测试账号信息:${NC}"
echo -e "  用户名: ${GREEN}demo_user${NC}"
echo -e "  密码:   ${GREEN}Demo123456${NC}"
echo -e "  租户:   ${GREEN}default${NC}"
echo -e "\n${BLUE}🔑 Access Token (24小时有效):${NC}"
echo -e "  ${TOKEN}"
echo -e "\n${BLUE}📊 已创建的测试数据:${NC}"
echo -e "  ✓ 用户: demo_user"
echo -e "  ✓ 角色: finance_manager (如果创建成功)"
echo -e "  ✓ 会计分录: 2 条"
echo -e "\n${BLUE}🌐 Swagger UI:${NC}"
echo -e "  ${GREEN}http://localhost:8081${NC}"
echo -e "\n${BLUE}💡 使用说明:${NC}"
echo -e "  1. 打开 Swagger UI: http://localhost:8081"
echo -e "  2. 选择 'CUBA ERP - 统一API'"
echo -e "  3. 点击右上角 'Authorize' 按钮"
echo -e "  4. 输入上面的 Token (包含 'Bearer ' 前缀)"
echo -e "  5. 现在可以测试所有接口了!"
echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
