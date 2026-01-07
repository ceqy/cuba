#!/bin/bash

# Configuration
API_URL="http://localhost:8050/api/v1"
ADMIN_USER="admin"
ADMIN_PASS="Password123!"

echo "--- 1. Login to get Token ---"
LOGIN_RESP=$(curl -s -X POST "$API_URL/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$ADMIN_USER\", \"password\": \"$ADMIN_PASS\"}")

TOKEN=$(echo $LOGIN_RESP | jq -r '.access_token')

if [ "$TOKEN" == "null" ]; then
  echo "Login failed: $LOGIN_RESP"
  exit 1
fi
echo "Login success. Token: ${TOKEN:0:10}..."

echo -e "\n--- 2. Enable 2FA ---"
ENABLE_RESP=$(curl -s -X POST "$API_URL/auth/2fa/enable" \
  -H "Authorization: Bearer $TOKEN")

SECRET=$(echo $ENABLE_RESP | jq -r '.secret_key')
echo "Secret Key: $SECRET"

if [ "$SECRET" == "null" ]; then
  echo "Enable 2FA failed: $ENABLE_RESP"
  exit 1
fi

echo -e "\n--- 3. Verify 2FA Setup (Mock Code) ---"
# Note: Real TOTP verification requires generating a valid code based on SECRET.
# For manual test without TOTP generator in bash, we might fail here unless we use a fixed mock secret or library.
# However, the goal is to verify endpoints are reachable.
# Let's try sending a dummy code, expecting 401 or failure, which confirms the endpoint works.
VERIFY_RESP=$(curl -s -X POST "$API_URL/auth/2fa/verify" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"code\": \"123456\"}")

echo "Verify Response: $VERIFY_RESP"

echo -e "\n--- 4. Verify 2FA Login (with Temp Token) ---"
# This requires a user WHO HAS 2FA enabled to login first to get temp_token.
# Since we just enabled it (but maybe didn't verify it successfully), the user state might not be 2FA enabled yet.
# But we can check the endpoint existence.

LOGIN_2FA_RESP=$(curl -s -X POST "$API_URL/auth/2fa/login" \
  -H "Content-Type: application/json" \
  -d "{\"temp_token\": \"dummy_temp_token\", \"code\": \"123456\"}")
echo "Login 2FA Response: $LOGIN_2FA_RESP"

echo -e "\nDone."
