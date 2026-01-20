#!/usr/bin/env bash
set -euo pipefail

echo "🚀 生成 OpenAPI 文档..."

# 确保输出目录存在
mkdir -p docs/openapi/generated

# 使用 buf 生成 OpenAPI 文档
buf generate

echo "✅ OpenAPI 文档已生成到: docs/openapi/generated/cuba-erp-api.swagger.json"
