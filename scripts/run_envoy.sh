#!/bin/bash
# Run Envoy gRPC-JSON Transcoder locally with Docker

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "Starting auth-service..."
cd "$PROJECT_ROOT"

# Kill existing processes
pkill -f "auth-service" || true
pkill -f "envoy" || true

# Start auth-service in background
cargo run -p auth-service &
AUTH_PID=$!

# Wait for auth-service to be ready
echo "Waiting for auth-service to start..."
sleep 5

# Check if auth-service is running
if ! kill -0 $AUTH_PID 2>/dev/null; then
    echo "Error: auth-service failed to start"
    exit 1
fi

echo "Starting Envoy proxy..."

# Run Envoy with Docker
docker run --rm -d \
  --name envoy-transcoder \
  -v "$PROJECT_ROOT/protos/auth/auth_service.pb:/etc/envoy/auth_service.pb:ro" \
  -v "$PROJECT_ROOT/deployments/envoy/envoy.yaml:/etc/envoy/envoy.yaml:ro" \
  -p 8080:8080 \
  -p 9901:9901 \
  envoyproxy/envoy:v1.28-latest \
  -c /etc/envoy/envoy.yaml

echo ""
echo "================================================================"
echo "Services are running!"
echo "================================================================"
echo "Auth Service (gRPC): http://localhost:50051"
echo "Envoy Proxy (HTTP):  http://localhost:8080"
echo "Envoy Admin:         http://localhost:9901"
echo ""
echo "Example API calls:"
echo "  curl http://localhost:8080/health"
echo "  curl -X POST http://localhost:8080/api/v1/auth/login -H 'Content-Type: application/json' -d '{\"username\":\"admin\",\"password\":\"Password123!\"}'"
echo ""
echo "Press Ctrl+C to stop."
echo "================================================================"

# Wait for interrupt
wait $AUTH_PID
