#!/bin/bash
# 一键启动所有服务

set -e
cd "$(dirname "$0")/.."

echo "🚀 Starting services..."

# Stop existing
docker stop swagger-ui envoy-transcoder 2>/dev/null || true
pkill -f auth-service 2>/dev/null || true

# 1. Auth Service
echo "  [1/3] Starting Auth Service..."
SERVER_ADDR="0.0.0.0:50051" cargo run -p auth-service &
sleep 3

# 2. Envoy
echo "  [2/3] Starting Envoy Transcoder..."
docker run --rm -d --name envoy-transcoder \
  -v "$(pwd)/protos/auth/auth_service.pb:/etc/envoy/auth_service.pb:ro" \
  -v "$(pwd)/deployments/envoy/envoy.yaml:/etc/envoy/envoy.yaml:ro" \
  -p 8080:8080 -p 9901:9901 \
  envoyproxy/envoy:v1.28-latest -c /etc/envoy/envoy.yaml

# 3. Swagger UI
echo "  [3/3] Starting Swagger UI..."
docker run --rm -d --name swagger-ui \
  -p 8081:8080 \
  -e SWAGGER_JSON=/docs/auth_service.swagger.json \
  -v "$(pwd)/docs/auth:/docs" \
  swaggerapi/swagger-ui

echo ""
echo "✅ All services started!"
echo "   API:     http://localhost:8080"
echo "   Swagger: http://localhost:8081"
