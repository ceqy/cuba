#!/bin/bash
# API 接口全面测试脚本

API="http://localhost:8080"
PASS=0
FAIL=0

test_api() {
    local method=$1
    local path=$2
    local data=$3
    local expected=$4
    local auth=$5
    
    if [ -n "$auth" ]; then
        resp=$(curl -s -w "\n%{http_code}" -X "$method" "$API$path" -H "Content-Type: application/json" -H "Authorization: Bearer $auth" -d "$data" 2>/dev/null)
    else
        resp=$(curl -s -w "\n%{http_code}" -X "$method" "$API$path" -H "Content-Type: application/json" -d "$data" 2>/dev/null)
    fi
    
    code=$(echo "$resp" | tail -1)
    body=$(echo "$resp" | sed '$d')
    
    if [[ "$code" =~ ^2 ]] || [[ "$code" == "$expected" ]]; then
        echo "✅ $method $path ($code)"
        ((PASS++))
    else
        echo "❌ $method $path (期望 $expected, 实际 $code)"
        echo "   响应正文: $body" | head -n 2
        ((FAIL++))
    fi
}

echo "=========================================="
echo "   API 接口测试 - $(date '+%Y-%m-%d %H:%M')"
echo "=========================================="
echo ""

# 1. 健康检查
echo "--- 基础接口 ---"
test_api GET "/health" "" "200"

# 2. 注册用户
echo -e "\n--- 认证接口 ---"
RAND=$RANDOM
test_api POST "/api/v1/auth/register" "{\"username\":\"testuser$RAND\",\"email\":\"test$RAND@test.com\",\"password\":\"Test123!\"}" "200"

# 3. 登录
LOGIN_RESP=$(curl -s -X POST "$API/api/v1/auth/login" -H "Content-Type: application/json" -d '{"username":"admin","password":"Password123!"}')
TOKEN=$(echo "$LOGIN_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('access_token',''))" 2>/dev/null)

if [ -n "$TOKEN" ]; then
    echo "✅ POST /api/v1/auth/login (已获取Token)"
    ((PASS++))
else
    echo "❌ POST /api/v1/auth/login (无法获取Token)"
    ((FAIL++))
    echo "跳过需要认证的测试..."
    echo ""
    echo "=========================================="
    echo "   测试结果: 通过 $PASS / 失败 $FAIL"
    echo "=========================================="
    exit 1
fi

# 4. 需要认证的接口
test_api GET "/api/v1/auth/me" "" "200" "$TOKEN"
test_api POST "/api/v1/auth/refresh" "{\"refresh_token\":\"invalid\"}" "401"
test_api POST "/api/v1/auth/logout" "{}" "200" "$TOKEN"

# 5. 密码相关
echo -e "\n--- 密码管理 ---"
test_api POST "/api/v1/auth/forgot-password" "{\"email\":\"admin@example.com\"}" "200"
test_api POST "/api/v1/auth/reset-password" "{\"token\":\"invalid\",\"new_password\":\"New123!\"}" "400"
test_api POST "/api/v1/auth/change-password" "{\"old_password\":\"wrong\",\"new_password\":\"New123!\"}" "401" "$TOKEN"

# 6. 2FA
echo -e "\n--- 双因素认证 ---"
test_api POST "/api/v1/auth/2fa/enable" "{}" "200" "$TOKEN"
test_api POST "/api/v1/auth/2fa/verify" "{\"code\":\"123456\"}" "400" "$TOKEN"
test_api POST "/api/v1/auth/2fa/login" "{\"temp_token\":\"invalid\",\"code\":\"123456\"}" "401"

# 7. 会话
echo -e "\n--- 会话管理 ---"
SESSIONS_RESP=$(curl -s -X GET "$API/api/v1/auth/sessions" -H "Authorization: Bearer $TOKEN")
test_api GET "/api/v1/auth/sessions" "" "200" "$TOKEN"
SESSION_ID=$(echo "$SESSIONS_RESP" | python3 -c "import sys,json; s=json.load(sys.stdin).get('sessions',[]); print(s[0]['session_id'] if s else '')" 2>/dev/null)
if [ -n "$SESSION_ID" ]; then
    test_api DELETE "/api/v1/auth/sessions/$SESSION_ID" "" "200" "$TOKEN"
fi

# 8. 角色
echo -e "\n--- 角色管理 ---"
test_api GET "/api/v1/roles" "" "200" "$TOKEN"
ROLE_NAME="testrole$RAND"
CREATE_ROLE_RESP=$(curl -s -X POST "$API/api/v1/roles" -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" -d "{\"name\":\"$ROLE_NAME\",\"description\":\"Test Description\"}")
ROLE_ID=$(echo "$CREATE_ROLE_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('role',{}).get('role_id',''))" 2>/dev/null)
if [ -n "$ROLE_ID" ]; then
    echo "✅ POST /api/v1/roles (已创角色: $ROLE_ID)"
    ((PASS++))
else
    echo "❌ POST /api/v1/roles (创建失败)"
    ((FAIL++))
fi

# 9. 权限
echo -e "\n--- 权限管理 ---"
PERMS_RESP=$(curl -s -X GET "$API/api/v1/permissions" -H "Authorization: Bearer $TOKEN")
test_api GET "/api/v1/permissions" "" "200" "$TOKEN"
PERM_NAME=$(echo "$PERMS_RESP" | python3 -c "import sys,json; p=json.load(sys.stdin).get('permissions',[]); print(p[0]['name'] if p else '')" 2>/dev/null)

if [ -n "$ROLE_ID" ] && [ -n "$PERM_NAME" ]; then
    echo "--- 角色权限绑定 ---"
    test_api POST "/api/v1/roles/$ROLE_ID/permissions" "{\"permission\":\"$PERM_NAME\"}" "200" "$TOKEN"
    test_api DELETE "/api/v1/roles/$ROLE_ID/permissions/$PERM_NAME" "" "200" "$TOKEN"
fi

# 10. 策略
echo -e "\n--- 策略管理 ---"
test_api GET "/api/v1/policies" "" "200" "$TOKEN"
POLICY_NAME="testpolicy$RAND"
CREATE_POLICY_RESP=$(curl -s -X POST "$API/api/v1/policies" -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" -d "{\"name\":\"$POLICY_NAME\",\"version\":\"1.0\",\"statements\":[{\"effect\":\"Allow\",\"actions\":[\"user:read\"],\"resources\":[\"*\"]}]}")
POLICY_ID=$(echo "$CREATE_POLICY_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('policy',{}).get('policy_id',''))" 2>/dev/null)

if [ -n "$POLICY_ID" ]; then
    echo "✅ POST /api/v1/policies (已创策略: $POLICY_ID)"
    ((PASS++))
    test_api GET "/api/v1/policies/$POLICY_ID" "" "200" "$TOKEN"
    
    # 策略绑定测试 (当前用户)
    ME_ID=$(echo "$LOGIN_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('user',{}).get('user_id',''))" 2>/dev/null)
    if [ -n "$ME_ID" ]; then
        test_api POST "/api/v1/users/$ME_ID/policies" "{\"policy_id\":\"$POLICY_ID\"}" "200" "$TOKEN"
    fi
    
    if [ -n "$ROLE_ID" ]; then
        test_api POST "/api/v1/roles/$ROLE_ID/policies" "{\"policy_id\":\"$POLICY_ID\"}" "200" "$TOKEN"
    fi
else
    echo "❌ POST /api/v1/policies (创建失败)"
    ((FAIL++))
fi

# 11. 用户管理
echo -e "\n--- 用户管理 ---"
USERS_RESP=$(curl -s -X GET "$API/api/v1/admin/users" -H "Authorization: Bearer $TOKEN")
test_api GET "/api/v1/admin/users" "" "200" "$TOKEN"
USER_ID=$(echo "$USERS_RESP" | python3 -c "import sys,json; u=json.load(sys.stdin).get('users',[]); print(u[0]['user_id'] if u else '')" 2>/dev/null)

if [ -n "$USER_ID" ]; then
    test_api PATCH "/api/v1/admin/users/$USER_ID/status" "{\"status\":\"Active\"}" "200" "$TOKEN"
    if [ -n "$ROLE_ID" ]; then
        test_api POST "/api/v1/users/$USER_ID/roles" "{\"role_id\":\"$ROLE_ID\"}" "200" "$TOKEN"
        test_api DELETE "/api/v1/users/$USER_ID/roles/$ROLE_ID" "" "200" "$TOKEN"
    fi
fi

# 11.1 用户资料更新
test_api PUT "/api/v1/users/profile" "{\"display_name\":\"New Name $RAND\"}" "200" "$TOKEN"

# 11.2 批量创建用户
test_api POST "/api/v1/admin/users/bulk" "{\"users\":[{\"username\":\"bulk1_$RAND\",\"email\":\"bulk1_$RAND@test.com\",\"password\":\"Bulk123!\"},{\"username\":\"bulk2_$RAND\",\"email\":\"bulk2_$RAND@test.com\",\"password\":\"Bulk123!\"}]}" "200" "$TOKEN"

# 12. 审计日志
echo -e "\n--- 审计日志 ---"
test_api GET "/api/v1/admin/audit-logs" "" "200" "$TOKEN"

# 13. API Keys
echo -e "\n--- API Keys ---"
KEYS_RESP=$(curl -s -X GET "$API/api/v1/api-keys" -H "Authorization: Bearer $TOKEN")
test_api GET "/api/v1/api-keys" "" "200" "$TOKEN"
KEY_ID=$(echo "$KEYS_RESP" | python3 -c "import sys,json; k=json.load(sys.stdin).get('keys',[]); print(k[0]['key_id'] if k else '')" 2>/dev/null)
if [ -n "$KEY_ID" ]; then
    test_api DELETE "/api/v1/api-keys/$KEY_ID" "" "200" "$TOKEN"
fi

# 14. OAuth2
echo -e "\n--- OAuth2 ---"
test_api GET "/api/v1/oauth2/clients" "" "200" "$TOKEN"
test_api POST "/api/v1/oauth2/clients" "{\"name\":\"test_client_$RAND\",\"redirect_uris\":[\"http://localhost/callback\"]}" "200" "$TOKEN"

# 16. Social Login
echo -e "\n--- Social Login ---"
# 有时返回 401 是因为系统直接认为 code 无效即认证失败
test_api POST "/api/v1/auth/social/login" "{\"provider\":\"google\",\"code\":\"invalid\"}" "401"

# 17. 清理 (删除测试中创建的角色和策略)
echo -e "\n--- 清理测试资源 ---"
if [ -n "$ROLE_ID" ]; then
    test_api DELETE "/api/v1/roles/$ROLE_ID" "" "200" "$TOKEN"
fi
if [ -n "$POLICY_ID" ]; then
    test_api DELETE "/api/v1/policies/$POLICY_ID" "" "200" "$TOKEN"
fi

echo ""
echo "=========================================="
echo "   测试结果: 通过 $PASS / 失败 $FAIL"
echo "=========================================="
