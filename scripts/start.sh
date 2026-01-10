#!/bin/bash
# 一键启动所有服务

set -e
cd "$(dirname "$0")/.."

# Source configuration
if [ -f "./scripts/config.sh" ]; then
    source ./scripts/config.sh
else
    echo "Error: ./scripts/config.sh not found."
    exit 1
fi

log_info "🚀 Starting services..."

# Stop existing containers/processes
log_info "Stopping existing services..."
docker stop swagger-ui envoy-transcoder 2>/dev/null || true
pkill -f auth-service 2>/dev/null || true

# 1. Auth Service
log_info "[1/3] Starting Auth Service..."
# Note: Ensure AUTH_SERVICE_PORT matches what the service expects
SERVER_ADDR="0.0.0.0:${AUTH_SERVICE_PORT}" cargo run -p auth-service > /dev/null 2>&1 &
wait_for_port "${AUTH_SERVICE_PORT}" "Auth Service"

# 2. Envoy
log_info "[2/3] Starting Envoy Transcoder..."
docker run --rm -d --name envoy-transcoder \
  -v "$(pwd)/protos/combined_services.pb:/etc/envoy/combined_services.pb:ro" \
  -v "$(pwd)/deployments/envoy/envoy.yaml:/etc/envoy/envoy.yaml:ro" \
  -p "${ENVOY_HTTP_PORT}:${ENVOY_HTTP_PORT}" \
  -p "${ENVOY_ADMIN_PORT}:${ENVOY_ADMIN_PORT}" \
  "${ENVOY_IMAGE}" -c /etc/envoy/envoy.yaml > /dev/null
wait_for_port "${ENVOY_HTTP_PORT}" "Envoy"

# 3. Swagger UI
log_info "[3/3] Starting Swagger UI..."

# Dynamic Swagger URL discovery
# Finds all .openapi3.json files in docs/ and formats them for Swagger UI's URLS env var
# Format: { "url": "/docs/auth/auth_service.openapi3.json", "name": "Auth Service" }
URLS_JSON="["
FIRST=true

while IFS= read -r file; do
    # Strip "./" prefix
    REL_PATH="${file#./}"
    # Generate a readable name from the filename or directory
    # e.g., docs/finance/gl/gl_journal_entry.openapi3.json -> Gl Journal Entry
    FILENAME=$(basename "$file" .openapi3.json)
    # Simple name cleanup: replace underscores with spaces, capitalize words
    NAME=$(echo "$FILENAME" | sed -r 's/[_-]+/ /g' | awk '{for(i=1;i<=NF;i++)sub(/./,toupper(substr($i,1,1)),$i)}1')
    
    if [ "$FIRST" = true ]; then
        FIRST=false
    else
        URLS_JSON="${URLS_JSON}, "
    fi
    URLS_JSON="${URLS_JSON}{ \"url\": \"/${REL_PATH}\", \"name\": \"${NAME}\" }"
done < <(find ./docs -name "*.openapi3.json" | sort)

URLS_JSON="${URLS_JSON}]"

docker run --rm -d --name swagger-ui \
  -p "${SWAGGER_UI_PORT}:8080" \
  -e URLS="${URLS_JSON}" \
  -e VALIDATOR_URL=none \
  -e PERSIST_AUTHORIZATION=true \
  -v "$(pwd)/docs:/usr/share/nginx/html/docs" \
  "${SWAGGER_UI_IMAGE}" > /dev/null

log_success "✅ All services started!"
echo ""
echo -e "   ${BLUE}API:${NC}     http://localhost:${ENVOY_HTTP_PORT}"
echo -e "   ${BLUE}Swagger:${NC} http://localhost:${SWAGGER_UI_PORT}"
