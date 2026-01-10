#!/bin/bash

# Project Configuration

# Network Ports
export AUTH_SERVICE_PORT="50051"
export GL_SERVICE_PORT="50052" # Assuming standard increment, currently unused in scripts but good to have
export ENVOY_HTTP_PORT="8080"
export ENVOY_ADMIN_PORT="9901"
export SWAGGER_UI_PORT="8081"
export API_GATEWAY_PORT="8050" # Rust API Gateway

# Docker Images
export ENVOY_IMAGE="envoyproxy/envoy:v1.28-latest"
export SWAGGER_UI_IMAGE="swaggerapi/swagger-ui:v5.31.0"

# Colors for Output
export GREEN='\033[0;32m'
export BLUE='\033[0;34m'
export RED='\033[0;31m'
export YELLOW='\033[1;33m'
export NC='\033[0m' # No Color

# Helper Functions
log_info() {
    echo -e "${BLUE}[INFO] $1${NC}"
}

log_success() {
    echo -e "${GREEN}[SUCCESS] $1${NC}"
}

log_warn() {
    echo -e "${YELLOW}[WARN] $1${NC}"
}

log_error() {
    echo -e "${RED}[ERROR] $1${NC}"
}

wait_for_port() {
    local PORT=$1
    local NAME=$2
    local RETRIES=30
    local WAIT_TIME=1

    echo -n "Waiting for $NAME on port $PORT..."
    for ((i=0; i<RETRIES; i++)); do
        if nc -z localhost $PORT 2>/dev/null; then
            echo -e " ${GREEN}Ready!${NC}"
            return 0
        fi
        echo -n "."
        sleep $WAIT_TIME
    done
    echo -e " ${RED}Timeout!${NC}"
    return 1
}
