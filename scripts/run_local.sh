#!/bin/bash
set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Kill existing processes on exit
cleanup() {
    echo -e "\n${BLUE}Stopping services...${NC}"
    # Kill all child processes in the current process group
    pkill -P $$ || true
}
trap cleanup SIGINT SIGTERM EXIT

echo -e "${BLUE}Building services...${NC}"
cargo build -p auth-service
cargo build -p api-gateway

echo -e "${BLUE}Starting Auth Service...${NC}"
./target/debug/auth-service &
AUTH_PID=$!
sleep 3 # Wait for startup

echo -e "${BLUE}Starting API Gateway...${NC}"
export AUTH_SERVICE_URL="http://[::1]:50051"
export GATEWAY_PORT="8050"
export RUST_LOG="api_gateway=debug,tower_http=debug"
./target/debug/api-gateway &
GATEWAY_PID=$!
sleep 2

echo -e "${GREEN}Services are running!${NC}"
echo -e "----------------------------------------------------------------"
echo -e "Swagger UI:   http://127.0.0.1:8050/swagger-ui"
echo -e "OpenAPI JSON: http://127.0.0.1:8050/api-docs/openapi.json"
echo -e "API Base URL: http://127.0.0.1:8050/api/v1"
echo -e "----------------------------------------------------------------"
echo -e "${BLUE}Press Ctrl+C to stop.${NC}"

# Wait for any process to exit
wait
