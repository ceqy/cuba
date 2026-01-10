#!/bin/bash
# 启动 Swagger UI 展示 CUBA ERP API 文档
# 用法: ./scripts/start-swagger.sh

cd "$(dirname "$0")/.."

# 停止旧容器
docker rm -f swagger-ui-preview 2>/dev/null

# 启动新容器
docker run -d --name swagger-ui-preview -p 8086:8080 \
  -v "$(pwd)/docs/openapi/splits:/usr/share/nginx/html/specs" \
  -e "DOC_EXPANSION=none" \
  -e "DEFAULT_MODELS_EXPAND_DEPTH=-1" \
  -e "URLS=[ \
    { url: './specs/finance.json', name: '财务 (Finance)' }, \
    { url: './specs/procurement.json', name: '采购 (Procurement)' }, \
    { url: './specs/sales.json', name: '销售 (Sales)' }, \
    { url: './specs/supplychain.json', name: '供应链 (Supply Chain)' }, \
    { url: './specs/asset.json', name: '资产管理 (Asset)' }, \
    { url: './specs/manufacturing.json', name: '制造 (Manufacturing)' }, \
    { url: './specs/service.json', name: '客户服务 (Service)' }, \
    { url: './specs/rd.json', name: '研发 (R&D)' }, \
    { url: './specs/hr.json', name: '人力资源 (HR)' }, \
    { url: './specs/auth.json', name: '身份认证 (IAM)' } \
  ]" \
  swaggerapi/swagger-ui

echo "✅ Swagger UI 已启动: http://localhost:8086"
