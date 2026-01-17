#!/bin/bash
# Script to generate secure secrets for Cuba platform
# Usage: ./generate-secrets.sh

set -e

echo "🔐 Generating secure secrets for Cuba platform..."
echo ""

# Generate PostgreSQL password
POSTGRES_PASSWORD=$(openssl rand -base64 24)
echo "✓ PostgreSQL password generated"

# Generate JWT secret key
JWT_SECRET=$(openssl rand -base64 32)
echo "✓ JWT secret key generated"

echo ""
echo "📝 Generated Secrets:"
echo "===================="
echo ""
echo "PostgreSQL Password:"
echo "$POSTGRES_PASSWORD"
echo ""
echo "JWT Secret Key:"
echo "$JWT_SECRET"
echo ""
echo "⚠️  IMPORTANT: Store these securely and update the Secret manifests!"
echo ""

# Optional: Create secrets directly in cluster
read -p "Do you want to create these secrets in Kubernetes now? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Creating secrets in Kubernetes..."
    
    # Create postgres secret in default namespace
    kubectl create secret generic cuba-postgres-credentials \
        --namespace=default \
        --from-literal=username=postgres \
        --from-literal=password="$POSTGRES_PASSWORD" \
        --from-literal=host=cuba-postgres.default.svc.cluster.local \
        --from-literal=port=5432 \
        --dry-run=client -o yaml | kubectl apply -f -
    
    echo "✓ Created cuba-postgres-credentials in default namespace"
    
    # Create postgres secret in cuba-infra namespace
    kubectl create secret generic cuba-postgres-credentials \
        --namespace=cuba-infra \
        --from-literal=username=postgres \
        --from-literal=password="$POSTGRES_PASSWORD" \
        --from-literal=host=postgres.cuba-infra.svc.cluster.local \
        --from-literal=port=5432 \
        --dry-run=client -o yaml | kubectl apply -f -
    
    echo "✓ Created cuba-postgres-credentials in cuba-infra namespace"
    
    # Create JWT secret in cuba-iam namespace
    kubectl create namespace cuba-iam --dry-run=client -o yaml | kubectl apply -f -
    kubectl create secret generic cuba-jwt-secret \
        --namespace=cuba-iam \
        --from-literal=secret-key="$JWT_SECRET" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    echo "✓ Created cuba-jwt-secret in cuba-iam namespace"
    echo ""
    echo "✅ All secrets created successfully!"
else
    echo "Skipped creating secrets in Kubernetes."
    echo "You can manually update the Secret manifests with these values."
fi
