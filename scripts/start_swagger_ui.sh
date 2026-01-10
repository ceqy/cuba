#!/bin/bash
# 启动 Swagger UI 展示所有服务的 API 文档 (动态扫描版)

set -e

DOCS_ROOT="$(pwd)/docs"

if [ ! -d "$DOCS_ROOT" ]; then
    echo "❌ Error: docs directory not found at $DOCS_ROOT"
    exit 1
fi

echo "🔍 Scanning for OpenAPI files in $DOCS_ROOT..."

# Initialize Arrays
URLS_JSON="["
DOCKER_VOLUMES=""
FIRST=true

# Find all .openapi3.json files
# Loop process: add to JSON array, add to Docker volume flags
while IFS= read -r file; do
    # Get relative path (e.g. finance/gl/gl_journal_entry.openapi3.json)
    REL_PATH=${file#$DOCS_ROOT/}
    
    # Get Filename (e.g. gl_journal_entry.openapi3.json)
    FILENAME=$(basename "$file")
    
    # Create a nice name (e.g. Gl Journal Entry)
    NAME=$(basename "$FILENAME" .openapi3.json | sed -r 's/_/ /g' | awk '{for(i=1;i<=NF;i++)sub(/./,toupper(substr($i,1,1)),$i)}1')
    
    echo "   Found: $FILENAME ($NAME)"

    # Add comma if not first
    if [ "$FIRST" = true ]; then
        FIRST=false
    else
        URLS_JSON+=","
    fi

    # Append to JSON
    # We serve files at /docs/ inside the container to avoid overwriting usage files
    # The mount will be: -v "$DOCS_ROOT:/usr/share/nginx/html/docs:ro"
    
    URLS_JSON+=" { \"url\": \"docs/$REL_PATH\", \"name\": \"$NAME\" }"
    
done < <(find "$DOCS_ROOT" -name "*.openapi3.json" | sort)

URLS_JSON+=" ]"

# Check if any files found
if [ "$FIRST" = true ]; then
    echo "⚠️  No OpenAPI files found. Did you run 'make openapi'?"
    # We continue anyway to show empty UI
fi

echo "🔄 Stopping existing Swagger UI..."
docker stop swagger-ui 2>/dev/null || true
docker rm swagger-ui 2>/dev/null || true

echo "🚀 Starting Swagger UI..."
docker run -d \
  --name swagger-ui \
  -p 8081:8080 \
  -e URLS="$URLS_JSON" \
  -e VALIDATOR_URL=none \
  -v "$DOCS_ROOT:/usr/share/nginx/html/docs:ro" \
  swaggerapi/swagger-ui:v5.31.0

echo ""
echo "✅ Swagger UI running at: http://localhost:8081"
echo "📂 Served files from: $DOCS_ROOT"
