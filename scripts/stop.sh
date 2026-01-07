#!/bin/bash
# 一键停止所有服务

echo "🛑 Stopping services..."
docker stop swagger-ui envoy-transcoder 2>/dev/null || true
pkill -f auth-service 2>/dev/null || true
echo "✅ All services stopped."
