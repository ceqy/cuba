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
    {url: 'http://localhost:8081/auth_service.openapi3.json', name: 'Auth Service (认证服务) - OpenAPI 3.0'},
    {url: 'http://localhost:8081/gl_journal_entry.openapi3.json', name: 'Finance Service (财务服务) - OpenAPI 3.0'}
  ]" \
  -v "$(pwd)/docs/auth/auth_service.openapi3.json:/usr/share/nginx/html/auth_service.openapi3.json:ro" \
  -v "$(pwd)/docs/finance/gl_journal_entry.openapi3.json:/usr/share/nginx/html/gl_journal_entry.openapi3.json:ro" \
  swaggerapi/swagger-ui

echo ""
echo "✅ Swagger UI 已启动！"
echo ""
echo "📖 API 文档地址："
echo "   http://localhost:8081"
echo ""
echo "📋 可用服务："
echo "   - Auth Service (认证服务)"
echo "   - Finance Service (财务服务)"
echo ""
echo "💡 提示：在 Swagger UI 右上角可以切换不同的服务文档"
