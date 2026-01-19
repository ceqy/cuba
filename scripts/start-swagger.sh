#!/bin/bash
# Start Swagger UI to display CUBA ERP API documentation
# Usage: ./scripts/start-swagger.sh

cd "$(dirname "$0")/.."

# Stop existing container
docker rm -f swagger-ui-preview 2>/dev/null

# Start new container
docker run -d --name swagger-ui-preview -p 8086:8080 \
  -v "$(pwd)/docs/openapi/splits:/usr/share/nginx/html/specs" \
  -e "DOC_EXPANSION=none" \
  -e "DEFAULT_MODELS_EXPAND_DEPTH=-1" \
  -e "URLS=[ \
    { url: './specs/finance.json', name: 'Finance (FI)' }, \
    { url: './specs/procurement.json', name: 'Procurement (PM)' }, \
    { url: './specs/sales.json', name: 'Sales (SD)' }, \
    { url: './specs/supplychain.json', name: 'Supply Chain (SC)' }, \
    { url: './specs/asset.json', name: 'Asset Management (AM)' }, \
    { url: './specs/manufacturing.json', name: 'Manufacturing (MF)' }, \
    { url: './specs/service.json', name: 'Customer Service (CS)' }, \
    { url: './specs/rd.json', name: 'R&D (RD)' }, \
    { url: './specs/hr.json', name: 'Human Resources (HR)' }, \
    { url: './specs/auth.json', name: 'Identity & Access (IAM)' } \
  ]" \
  swaggerapi/swagger-ui

echo "Swagger UI started: http://localhost:8086"
