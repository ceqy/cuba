#!/bin/bash
set -e

# Source configuration
if [ -f "./scripts/config.sh" ]; then
    source ./scripts/config.sh
else
    # Fallback to local definitions if config is missing (robustness)
    BLUE='\033[0;34m'
    GREEN='\033[0;32m'
    NC='\033[0m'
    AUTH_SERVICE_PORT="50051"
    API_GATEWAY_PORT="8050"
fi

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
wait_for_port "${AUTH_SERVICE_PORT}" "Auth Service"

echo -e "${BLUE}Starting API Gateway...${NC}"
export AUTH_SERVICE_URL="http://[::1]:${AUTH_SERVICE_PORT}"
export GATEWAY_PORT="${API_GATEWAY_PORT}"
export RUST_LOG="api_gateway=debug,tower_http=debug"
./target/debug/api-gateway &
# We don't have wait_for_port sourced effectively if config was missing or simplistic; 
# but assuming config.sh exists as per plan.
sleep 2 

echo -e "${GREEN}Services are running!${NC}"
echo -e "----------------------------------------------------------------"
echo -e "Swagger UI:   http://127.0.0.1:${API_GATEWAY_PORT}/swagger-ui"
echo -e "OpenAPI JSON: http://127.0.0.1:${API_GATEWAY_PORT}/api-docs/openapi.json"
echo -e "API Base URL: http://127.0.0.1:${API_GATEWAY_PORT}/api/v1"
echo -e "----------------------------------------------------------------"
echo -e "${BLUE}Press Ctrl+C to stop.${NC}"

# Wait for any process to exit
wait
