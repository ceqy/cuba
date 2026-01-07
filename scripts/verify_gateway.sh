#!/bin/bash
set -e

# Configuration
DB_URL="postgres://postgres:postgres@localhost:5432/postgres"
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

USER_ID=$(echo $ME_RES | grep -o '"user_id":"[^"]*' | cut -d'"' -f4)
if [ -z "$USER_ID" ]; then
    echo "Failed to get user info."
    exit 1
fi

echo ">>> Verification SUCCESS!"
