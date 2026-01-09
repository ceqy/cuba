#!/bin/bash
# 启动 Swagger UI 展示所有服务的 API 文档

set -e

echo "🔄 正在停止现有的 Swagger UI..."
docker stop swagger-ui 2>/dev/null || true
docker rm swagger-ui 2>/dev/null || true

echo "🚀 启动 Swagger UI (多服务支持)..."
docker run -d \
  --name swagger-ui \
  -p 8081:8080 \
  -e URLS="[
    { \"url\": \"auth_service.openapi3.json\", \"name\": \"Auth Service (认证服务)\" },
    { \"url\": \"gl_journal_entry.openapi3.json\", \"name\": \"GL Service (财务总账)\" },
    { \"url\": \"ar_ap.openapi3.json\", \"name\": \"AR/AP Service (应收应付)\" }
  ]" \
  -e VALIDATOR_URL=none \
  -v "$(pwd)/docs/auth/auth_service.openapi3.json:/usr/share/nginx/html/auth_service.openapi3.json:ro" \
  -v "$(pwd)/docs/finance/gl_journal_entry.openapi3.json:/usr/share/nginx/html/gl_journal_entry.openapi3.json:ro" \
  -v "$(pwd)/docs/finance/ar_ap.openapi3.json:/usr/share/nginx/html/ar_ap.openapi3.json:ro" \
  swaggerapi/swagger-ui:v5.31.0

echo ""
echo "✅ Swagger UI 已启动！"
echo ""
echo "📖 API 文档地址："
echo "   http://localhost:8081"
echo ""
echo "📋 可用服务："
echo "   - Auth Service (认证服务)"
echo "   - GL Service (财务总账)"
echo "   - AR/AP Service (应收应付)"
echo ""
echo "💡 提示：在 Swagger UI 右上角可以切换不同的服务文档"
