#!/bin/bash
# Cuba ERP - Istio Deployment Script

set -e

echo "=== Cuba ERP Istio Deployment ==="

# Check prerequisites
echo "Checking prerequisites..."
kubectl version --client > /dev/null 2>&1 || { echo "kubectl not found"; exit 1; }
istioctl version > /dev/null 2>&1 || { echo "istioctl not found"; exit 1; }

# Apply namespaces first
echo "Creating namespaces..."
kubectl apply -f deploy/k8s/infra/namespaces.yaml

# Wait for namespaces
sleep 2

# Apply Istio Gateway and VirtualService
echo "Deploying Istio Gateway and VirtualService..."
kubectl apply -f deploy/k8s/infra/istio-gateway.yaml

# Apply Authorization Policies
echo "Deploying Authorization Policies..."
kubectl apply -f deploy/k8s/infra/authorization-policies.yaml

# Deploy PostgreSQL
echo "Deploying PostgreSQL..."
kubectl apply -f deploy/k8s/infra/postgres.yaml

echo ""
echo "=== Deployment Summary ==="
echo "Namespaces:"
kubectl get ns | grep cuba

echo ""
echo "Istio Resources:"
kubectl get gateway,virtualservice -n cuba-system

echo ""
echo "=== Istio Configuration Complete ==="
echo ""
echo "Next steps:"
echo "1. Deploy individual services using Helm:"
echo "   helm install gl-service deploy/k8s/charts/microservice -f deploy/k8s/values/gl-service.yaml -n cuba-fi"
echo ""
echo "2. Verify Istio injection:"
echo "   kubectl get pods -n cuba-fi -o jsonpath='{.items[*].spec.containers[*].name}' | tr ' ' '\n' | grep istio-proxy"
