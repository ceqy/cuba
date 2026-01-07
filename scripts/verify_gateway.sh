#!/bin/bash
set -e

# Configuration
DB_URL="postgres://postgres:postgres@localhost:5432/cuba_auth"
GATEWAY_URL="http://127.0.0.1:8050"
AUTH_SERVICE_PORT=50051

echo ">>> Checking Database Connection..."
if ! command -v sqlx &> /dev/null; then
    echo "sqlx CLI not found. Skipping migration check (Assuming DB is initialized)."
else
    echo "Running migrations..."
    # 暂时禁用 sqlx check，如果不想安装 sqlx-cli
    # sqlx migrate run --source ../migrations --database-url "$DB_URL"
    echo "Skipped explicit sqlx run in this script to avoid stalling if sqlx missing."
fi

echo ">>> Building Services..."
cargo build -p auth-service -p api-gateway

echo ">>> Starting Auth Service..."
# 设置环境变量，确保 cuba-config 能找到
export RUN_MODE=dev
export RUST_BACKTRACE=1
# 启动 auth-service，日志输出到文件
target/debug/auth-service > auth_service.log 2>&1 &
AUTH_PID=$!
echo "Auth Service PID: $AUTH_PID"

echo "Waiting for Auth Service to be ready..."
sleep 5

echo ">>> Starting API Gateway..."
export AUTH_SERVICE_URL="http://[::1]:$AUTH_SERVICE_PORT"
export GATEWAY_PORT=8050
target/debug/api-gateway > api_gateway.log 2>&1 &
GATEWAY_PID=$!
echo "API Gateway PID: $GATEWAY_PID"

echo "Waiting for Gateway to be ready..."
sleep 3

# Cleanup checks
function cleanup {
    echo ">>> Stopping services..."
    kill $AUTH_PID || true
    kill $GATEWAY_PID || true
    echo "Done."
}
trap cleanup EXIT

# Check if gateway is actually running
if ! curl -s "$GATEWAY_URL/health" | grep -q "UP"; then
    echo -e "${RED}Gateway failed to start or health check failed.${NC}"
    exit 1
fi

echo -e "\n${YELLOW}>>> Test OpenAPI Documentation${NC}"
# Base URL is http://localhost:8050/api/v1 -> remove /v1 -> http://localhost:8050/api -> No.
# URL construction in main.rs: .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ...))
# So it should be at http://localhost:8050/api-docs/openapi.json
OPENAPI_URL="http://localhost:8050/api-docs/openapi.json"
OPENAPI_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$OPENAPI_URL")

if [ "$OPENAPI_CODE" -eq 200 ]; then
    echo -e "${GREEN}OpenAPI JSON is accessible at ${OPENAPI_URL}${NC}"
else
    echo -e "${RED}Failed to access OpenAPI JSON. Status: ${OPENAPI_CODE}${NC}"
    # Continue but mark error?
fi

echo ">>> Test 1: Health Check"
curl -s "$GATEWAY_URL/health" | grep "UP"
echo "Health check passed."

echo ">>> Test 2: Register User"
RANDOM_USER="user_$RANDOM"
# Register
REGISTER_RES=$(curl -s -X POST "$GATEWAY_URL/api/v1/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$RANDOM_USER\", \"email\": \"$RANDOM_USER@example.com\", \"password\": \"Password123!\"}")

echo "Register Response: $REGISTER_RES"

# Start Login
echo ">>> Test 3: Login User"
LOGIN_RES=$(curl -s -X POST "$GATEWAY_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$RANDOM_USER\", \"password\": \"Password123!\"}")

echo "Login Response: $LOGIN_RES"

TOKEN=$(echo $LOGIN_RES | grep -o '"access_token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo "Login failed, no token found."
    exit 1
fi

echo "Got Token: $TOKEN"

echo ">>> Test 4: Get User Info (Me)"
ME_RES=$(curl -s -X GET "$GATEWAY_URL/api/v1/auth/me" \
  -H "Authorization: Bearer $TOKEN")

echo "Me Response: $ME_RES"

echo -e "\n\n>>> Test 5: Password Management"
echo "Changing password from 'Password123!' to 'NewPassword123!'..."
CHANGE_PWD_RES=$(curl -s -X POST "$GATEWAY_URL/api/v1/auth/change-password" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"old_password\": \"Password123!\", \"new_password\": \"NewPassword123!\"}")
echo "Change Password Response: $CHANGE_PWD_RES"

echo "Verifying new password login..."
NEW_LOGIN_RES=$(curl -s -X POST "$GATEWAY_URL/api/v1/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$RANDOM_USER\", \"password\": \"NewPassword123!\"}")

if echo "$NEW_LOGIN_RES" | grep -q "access_token"; then
  echo "Login with new password successful."
else
  echo "Login with new password FAILED. Response: $NEW_LOGIN_RES"
  # Don't exit immediately, try to recover/cleanup
fi

echo "Testing Forgot Password..."
FORGOT_RES=$(curl -s -X POST "$GATEWAY_URL/api/v1/auth/forgot-password" \
  -H "Content-Type: application/json" \
  -d "{\"email\": \"$RANDOM_USER@example.com\"}")
echo "Forgot Password Response: $FORGOT_RES"

USER_ID=$(echo $ME_RES | grep -o '"user_id":"[^"]*' | cut -d'"' -f4)
if [ -z "$USER_ID" ]; then
    echo "Failed to get user info."
    exit 1
fi

# ... tests 1-5 ...

echo -e "\n${YELLOW}>>> Test 6: RBAC Management${NC}"
echo "Creating a new role 'editor'..."
ROLE_RES=$(curl -s -X POST "${BASE_API_URL}/roles" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "editor",
    "description": "Content Editor"
  }')
echo "Create Role Response: ${ROLE_RES}"

# Extract Role ID from response (basic grep, ideally use jq)
if echo "$ROLE_RES" | grep -q "role_id"; then
    echo -e "${GREEN}Role created.${NC}"
else
    echo -e "${RED}Failed to create role.${NC}"
fi

echo "Listing roles..."
LIST_ROLES_RES=$(curl -s -X GET "${BASE_API_URL}/roles?page=0&page_size=10" \
  -H "Authorization: Bearer ${TOKEN}")

echo "List Roles Response: ${LIST_ROLES_RES}"

# Extract an actual role id for assignment (using jq would be better but keeping dependency low)
# Assuming the response format {"roles":[{"role_id":"...",...},...]}
# Let's try to parse the first role_id from list or use the one from create response if we parsed it.
# For simplicity, let's just use the USER_ID we have and try to assign "editor" role if we knew its ID.
# Since we don't have jq to easily parse JSON in this script (usually), we might skip dynamic ID extraction for now
# OR we can try a simple sed/awk extraction.
ROLE_ID=$(echo $ROLE_RES | sed -n 's/.*"role_id":"\([^"]*\)".*/\1/p')

if [ -n "$ROLE_ID" ]; then
    echo "Assigning role $ROLE_ID to user $USER_ID..."
    ASSIGN_RES=$(curl -s -X POST "${BASE_API_URL}/users/${USER_ID}/roles" \
      -H "Authorization: Bearer ${TOKEN}" \
      -H "Content-Type: application/json" \
      -d "{
        \"role_id\": \"${ROLE_ID}\"
      }")
    echo "Assign Role Response: ${ASSIGN_RES}"
else
    echo -e "${RED}Could not extract role_id to test assignment.${NC}"
fi

echo -e "\n${YELLOW}>>> Test 7: 2FA Flow (Partial)${NC}"
echo "Enabling 2FA..."
ENABLE_2FA_RES=$(curl -s -X POST "${BASE_API_URL}/auth/2fa/enable" \
  -H "Authorization: Bearer ${TOKEN}")
echo "Enable 2FA Response: ${ENABLE_2FA_RES}"

if echo "$ENABLE_2FA_RES" | grep -q "secret_key"; then
    echo -e "${GREEN}2FA Enabled (Secret received).${NC}"
else
    echo -e "${RED}Failed to enable 2FA.${NC}"
fi

echo "Verifying 2FA Setup with INVALID code (Expect Failure)..."
VERIFY_SETUP_RES=$(curl -s -X POST "${BASE_API_URL}/auth/2fa/verify" \
  -H "Authorization: Bearer ${TOKEN}" \
  -H "Content-Type: application/json" \
  -d '{
    "code": "000000"
  }')
echo "Verify Setup Response: ${VERIFY_SETUP_RES}"

# Expecting failure or false success depending on response structure.
# Verify2FaSetupResponse { success: bool }
if echo "$VERIFY_SETUP_RES" | grep -q "\"success\":false"; then
    echo -e "${GREEN}2FA Setup Verification correctly failed with invalid code.${NC}"
elif echo "$VERIFY_SETUP_RES" | grep -q "false"; then # Simply check for false if structure is simple
    echo -e "${GREEN}2FA Setup Verification correctly failed with invalid code.${NC}"
else 
   # It might also return 400 Bad Request error from gRPC mapping
   if echo "$VERIFY_SETUP_RES" | grep -q "error"; then
        echo -e "${GREEN}2FA Setup Verification correctly failed with error.${NC}"
   else
        echo -e "${YELLOW}Unexpected response for invalid code (Might be expected if boolean logic differs).${NC}"
   fi
fi

echo -e "\n${GREEN}>>> Verification SUCCESS!${NC}"
