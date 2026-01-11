#!/bin/bash
set -e

CLUSTER_NAME="cuba-cluster"

echo "🚀 Starting Cuba ERP Kubernetes Deployment..."

# 1. Check Tools
command -v kind >/dev/null 2>&1 || { echo "❌ kind is required but not installed."; exit 1; }
command -v kubectl >/dev/null 2>&1 || { echo "❌ kubectl is required but not installed."; exit 1; }
command -v helm >/dev/null 2>&1 || { echo "❌ helm is required but not installed."; exit 1; }
command -v istioctl >/dev/null 2>&1 || { echo "❌ istioctl is required but not installed."; exit 1; }

# 2. Create Cluster
if kind get clusters | grep -q "^$CLUSTER_NAME$"; then
    echo "✅ Cluster '$CLUSTER_NAME' already exists."
else
    echo "📦 Creating Kind cluster '$CLUSTER_NAME'..."
    kind create cluster --name "$CLUSTER_NAME" --config - <<EOF
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
  kubeadmConfigPatches:
  - |
    kind: InitConfiguration
    nodeRegistration:
      kubeletExtraArgs:
        node-labels: "ingress-ready=true"
  extraPortMappings:
  - containerPort: 80
    hostPort: 80
    protocol: TCP
  - containerPort: 443
    hostPort: 443
    protocol: TCP
EOF
fi

# 3. Install Istio
echo "🕸️  Installing Istio..."
istioctl install --set profile=demo -y

# 4. Create Namespace & Enable Injection
echo "💉 Configuring Namespace..."
kubectl create namespace cuba --dry-run=client -o yaml | kubectl apply -f -
kubectl label namespace cuba istio-injection=enabled --overwrite
kubectl config set-context --current --namespace=cuba

# 5. Build & Load Images
SERVICES=("iam-service" "gl-service" "ap-service")
APPS_DIR="./apps"

# Postgres
echo "🐘 Deploying Postgres..."
kubectl apply -f deploy/k8s/infra/postgres.yaml

echo "🏗️  Building and Loading Images..."
for SVC in "${SERVICES[@]}"; do
    DOMAIN="unknown"
    if [[ "$SVC" == "iam-service" ]]; then DOMAIN="iam"; fi
    if [[ "$SVC" == "gl-service" ]]; then DOMAIN="fi"; fi
    if [[ "$SVC" == "ap-service" ]]; then DOMAIN="fi"; fi
    
    # Simple heuristic for dockerfile path
    DOCKERFILE="apps/$DOMAIN/$SVC/Dockerfile"
    
    IMAGE_NAME="cuba-$SVC:latest"
    echo "   Building $IMAGE_NAME from $DOCKERFILE..."
    docker build -t "$IMAGE_NAME" -f "$DOCKERFILE" .
    
    echo "   Loading $IMAGE_NAME into Kind..."
    kind load docker-image "$IMAGE_NAME" --name "$CLUSTER_NAME"
done

# 6. Deploy Services via Helm
echo "🚀 Deploying Microservices..."
for SVC in "${SERVICES[@]}"; do
    echo "   Installing $SVC..."
    helm upgrade --install "$SVC" deploy/k8s/charts/microservice \
        -f "deploy/k8s/values/$SVC.yaml" \
        --set image.tag=latest \
        --namespace cuba
done

# 7. Apply Istio Gateway
echo "🚪 Configuring Gateway..."
kubectl apply -f deploy/k8s/infra/istio-gateway.yaml

echo "🎉 Deployment Complete!"
echo "   Run 'kubectl get pods -n cuba' to see status."
echo "   Access services at http://localhost/api/v1/..."
