#!/bin/bash
# 为认证服务生成 OpenAPI 3.0 文档

set -e

echo "正在生成认证服务 OpenAPI 文档..."

# 生成 OpenAPI 2.0
protoc -I./protos -I./protos/third_party \
  --openapiv2_out=./docs \
  --openapiv2_opt logtostderr=true \
  --openapiv2_opt allow_delete_body=true \
  protos/auth/auth_service.proto

# 检查生成的文档并移动到预期位置
if [ -f "docs/auth/auth_service.swagger.json" ]; then
    echo "✅ 认证服务 OpenAPI 2.0 文档: docs/auth/auth_service.swagger.json"
    
    echo "🔄 正在转换为 OpenAPI 3.0..."
    npx -y swagger2openapi -o docs/auth/auth_service.openapi3.json docs/auth/auth_service.swagger.json
    
    echo "🏷️  正在添加标签描述..."
    python3 scripts/add_tag_descriptions.py auth
    
    echo "✅ 认证服务 OpenAPI 3.0 文档: docs/auth/auth_service.openapi3.json"
    ls -lh docs/auth/auth_service.openapi3.json
else
    echo "❌ 文档生成失败"
    exit 1
fi
