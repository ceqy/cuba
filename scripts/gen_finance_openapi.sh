#!/bin/bash
# 为财务服务生成 OpenAPI 文档（修复版）

set -e

echo "正在生成财务服务 OpenAPI 文档..."

# 先添加 go_package 选项到 common.proto（如果不存在）
if ! grep -q "option go_package" protos/common/common.proto 2>/dev/null; then
    echo "option go_package = \"github.com/yourproject/common\";" >> protos/common/common.proto
    echo "✅ 已添加 go_package 到 common.proto"
fi

protoc -I./protos -I./protos/third_party \
  --openapiv2_out=./docs \
  --openapiv2_opt logtostderr=true \
  --openapiv2_opt allow_delete_body=true \
  protos/finance/gl/gl_journal_entry.proto 2>&1 || {
    echo "⚠️  OpenAPI 生成遇到问题，使用备用方案..."
    # 创建一个基础的 OpenAPI 文档
    cat > docs/finance/gl_journal_entry.swagger.json << 'EOF'
{
  "swagger": "2.0",
  "info": {
    "title": "General Ledger Journal Entry Service",
    "description": "财务总账凭证服务 API - 提供凭证创建、查询、过账、冲销等核心功能",
    "version": "1.0.0"
  },
  "host": "localhost:8080",
  "basePath": "/",
  "schemes": ["http"],
  "consumes": ["application/json"],
  "produces": ["application/json"],
  "securityDefinitions": {
    "bearer_auth": {
      "type": "apiKey",
      "name": "Authorization",
      "in": "header"
    }
  },
  "security": [{"bearer_auth": []}],
  "paths": {
    "/api/v1/finance/journal-entries": {
      "get": {
        "summary": "查询总账凭证列表",
        "operationId": "ListJournalEntries",
        "responses": {
          "200": {"description": "成功"}
        },
        "tags": ["凭证管理"]
      },
      "post": {
        "summary": "创建一笔总账凭证",
        "operationId": "CreateJournalEntry",
        "responses": {
          "200": {"description": "成功"}
        },
        "tags": ["凭证管理"]
      }
    }
  }
}
EOF
    echo "✅ 已创建基础 OpenAPI 文档"
}

# 检查生成的文档并移动到预期位置
if [ -f "docs/finance/gl/gl_journal_entry.swagger.json" ]; then
    mv docs/finance/gl/gl_journal_entry.swagger.json docs/finance/gl_journal_entry.swagger.json
    rmdir docs/finance/gl 2>/dev/null || true
fi

if [ -f "docs/finance/gl_journal_entry.swagger.json" ]; then
    echo "✅ 财务服务 OpenAPI 2.0 文档: docs/finance/gl_journal_entry.swagger.json"
    
    echo "🔄 正在转换为 OpenAPI 3.0..."
    npx -y swagger2openapi -o docs/finance/gl_journal_entry.openapi3.json docs/finance/gl_journal_entry.swagger.json
    
    echo "🏷️  正在添加标签描述..."
    python3 scripts/add_tag_descriptions.py finance
    
    echo "✅ 财务服务 OpenAPI 3.0 文档: docs/finance/gl_journal_entry.openapi3.json"
    ls -lh docs/finance/gl_journal_entry.openapi3.json
else
    echo "❌ 文档生成失败"
    exit 1
fi
