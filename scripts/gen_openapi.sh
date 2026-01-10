#!/bin/bash
# Universal OpenAPI Generator Script
# Usage: ./scripts/gen_openapi.sh [service_path]
# Example: ./scripts/gen_openapi.sh finance/gl
# If no argument is provided, lists available services.

set -e

# Base directories
PROTOS_DIR="./protos"
DOCS_DIR="./docs"

# Function to generate OpenAPI for a single service
generate_service() {
    local SERVICE_PATH=$1
    local SERVICE_NAME=$(basename $SERVICE_PATH)
    
    # Find .proto files in the directory
    local PROTO_FILES=$(find "$PROTOS_DIR/$SERVICE_PATH" -maxdepth 1 -name "*.proto")
    
    if [ -z "$PROTO_FILES" ]; then
        echo "❌ No .proto files found in $PROTOS_DIR/$SERVICE_PATH"
        return 1
    fi

    echo "🚀 Generating OpenAPI for: $SERVICE_PATH"

    # Create output directory
    local OUT_DIR="$DOCS_DIR/$SERVICE_PATH"
    mkdir -p "$OUT_DIR"

    for PROTO_FILE in $PROTO_FILES; do
        local FILENAME=$(basename "$PROTO_FILE" .proto)
        echo "   Processing $FILENAME.proto..."

        # Calculate relative path for M flag (strip ./protos/)
        local REL_PATH=${PROTO_FILE#./protos/}
        
        # Generate OpenAPI 2.0 (Swagger)
        # Note: We include third_party and current dir as include paths
        protoc -I"$PROTOS_DIR" -I"$PROTOS_DIR/third_party" \
            --openapiv2_out="$OUT_DIR" \
            --openapiv2_opt=logtostderr=true \
            --openapiv2_opt=allow_delete_body=true \
            --openapiv2_opt=json_names_for_fields=false \
            --openapiv2_opt=M${REL_PATH}=github.com/enterprise/generated/${SERVICE_PATH} \
            "$PROTO_FILE"
            
        # Handle potential nested directory output (e.g., docs/finance/gl/finance/gl/...)
        local SWAGGER_FILE="$OUT_DIR/$FILENAME.swagger.json"
        
        # Check if protoc generated it inside a nested structure
        # Use -mindepth 2 to ignore the file if it's already in the root of OUT_DIR
        local NESTED_FILE=$(find "$OUT_DIR" -mindepth 2 -name "$FILENAME.swagger.json" | head -n 1)
        if [ -n "$NESTED_FILE" ]; then
             # Move strictly if it's nested
             mv "$NESTED_FILE" "$SWAGGER_FILE"
             # Try to clean up empty directories
             rmdir -p "$(dirname "$NESTED_FILE")" 2>/dev/null || true
        fi
        
        local OPENAPI3_FILE="$OUT_DIR/$FILENAME.openapi3.json"

        if [ -f "$SWAGGER_FILE" ]; then
             echo "   ✅ Generated Swagger 2.0: $SWAGGER_FILE"
             
             # Convert to OpenAPI 3.0
             echo "   🔄 Converting to OpenAPI 3.1..."
             npx -y swagger2openapi --targetVersion 3.1.0 -o "$OPENAPI3_FILE" "$SWAGGER_FILE" > /dev/null 2>&1
             
             if [ -f "$OPENAPI3_FILE" ]; then
                 echo "   ✅ Generated OpenAPI 3.1: $OPENAPI3_FILE"
                 
                 # Enrich OpenAPI (Add Security, Types, Descriptions)
                 echo "   ✨ Enriching OpenAPI spec..."
                 python3 ./scripts/enrich_openapi.py "$OPENAPI3_FILE"
             else
                 echo "   ⚠️ Failed to convert to OpenAPI 3.0"
             fi
        else
            echo "   ⚠️ Failed to generate Swagger 2.0 for $FILENAME"
        fi
    done
}

# Main Execution
if [ -z "$1" ]; then
    echo "Usage: $0 <service_path>"
    echo "Examples:"
    echo "  $0 finance/gl"
    echo "  $0 finance/ar_ap"
    echo "  $0 auth"
    exit 1
fi

generate_service "$1"
