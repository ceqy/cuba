#!/bin/bash
set -e

echo "=== Deploying IAM Microservices to Kubernetes ==="

# 1. Create namespace
echo "Creating cuba-iam namespace..."
kubectl create namespace cuba-iam --dry-run=client -o yaml | kubectl apply -f -
kubectl label namespace cuba-iam istio-injection=enabled --overwrite

# 2. Generate and apply proto descriptor ConfigMap
echo "Generating proto.pb descriptor..."
protoc --include_imports --include_source_info \
  --descriptor_set_out=/tmp/proto.pb \
  -I protos \
  -I third_party \
  protos/iam/auth/auth.proto \
  protos/iam/rbac/rbac.proto \
  protos/iam/oauth/oauth.proto \
  protos/common/common.proto \
  protos/common/types.proto

echo "Creating cuba-proto-descriptor ConfigMap..."
kubectl delete configmap cuba-proto-descriptor -n istio-system --ignore-not-found
kubectl create configmap cuba-proto-descriptor \
  --from-file=proto.pb=/tmp/proto.pb \
  -n istio-system

# 3. Deploy IAM services using Helm
echo "Deploying auth-service..."
helm upgrade --install auth-service deploy/k8s/charts/microservice \
  -f deploy/k8s/values/auth-service.yaml \
  --namespace cuba-iam \
  --wait --timeout 2m

echo "Deploying rbac-service..."
helm upgrade --install rbac-service deploy/k8s/charts/microservice \
  -f deploy/k8s/values/rbac-service.yaml \
  --namespace cuba-iam \
  --wait --timeout 2m

echo "Deploying oauth-service..."
helm upgrade --install oauth-service deploy/k8s/charts/microservice \
  -f deploy/k8s/values/oauth-service.yaml \
  --namespace cuba-iam \
  --wait --timeout 2m

# 4. Verify deployments
echo "Verifying deployments..."
kubectl get pods -n cuba-iam

echo "=== IAM Deployment Complete ==="
echo ""
echo "Services deployed:"
echo "  - auth-service:  50051"
echo "  - rbac-service:  50052"
echo "  - oauth-service: 50053"
