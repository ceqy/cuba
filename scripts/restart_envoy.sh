#!/bin/bash
# 重启 Envoy 并加载财务服务配置

set -e

echo "🔄 正在停止现有的 Envoy 容器..."
docker stop envoy 2>/dev/null || true
docker rm envoy 2>/dev/null || true

echo "🚀 启动 Envoy 代理（包含财务服务路由）..."
docker run -d \
  --name envoy \
  -p 8080:8080 \
  -p 9901:9901 \
  -v "$(pwd)/deployments/envoy/envoy.yaml:/etc/envoy/envoy.yaml:ro" \
  -v "$(pwd)/protos/combined_services.pb:/etc/envoy/combined_services.pb:ro" \
  envoyproxy/envoy:v1.28-latest

echo ""
echo "✅ Envoy 已启动！"
echo ""
echo "📋 服务映射："
echo "   - Auth Service:    POST/GET  http://localhost:8080/api/v1/auth/*"
echo "   - Finance Service: POST/GET  http://localhost:8080/api/v1/finance/*"
echo ""
echo "🔍 管理界面："
echo "   - Envoy Admin: http://localhost:9901"
echo ""
echo "💡 测试示例："
echo "   curl http://localhost:8080/health"
echo "   curl http://localhost:8080/api/v1/finance/journal-entries"
