#!/bin/bash
set -e

# Delete existing CMs
kubectl delete configmap cuba-docs-1 -n cuba-system --ignore-not-found
kubectl delete configmap cuba-docs-2 -n cuba-system --ignore-not-found
kubectl delete configmap cuba-docs-3 -n cuba-system --ignore-not-found
kubectl delete configmap cuba-docs-4 -n cuba-system --ignore-not-found
kubectl delete configmap cuba-docs-5 -n cuba-system --ignore-not-found

echo "Creating ConfigMaps..."

# Create directly
kubectl create configmap cuba-docs-1 -n cuba-system --from-file=docs/openapi/splits/auth.json --from-file=docs/openapi/splits/finance.json
kubectl create configmap cuba-docs-2 -n cuba-system --from-file=docs/openapi/splits/asset.json --from-file=docs/openapi/splits/procurement.json
kubectl create configmap cuba-docs-3 -n cuba-system --from-file=docs/openapi/splits/sales.json --from-file=docs/openapi/splits/service.json
kubectl create configmap cuba-docs-4 -n cuba-system --from-file=docs/openapi/splits/manufacturing.json --from-file=docs/openapi/splits/supplychain.json
kubectl create configmap cuba-docs-5 -n cuba-system --from-file=docs/openapi/splits/hr.json --from-file=docs/openapi/splits/rd.json

echo "ConfigMaps created."
