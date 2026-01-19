#!/bin/bash
# CUBA ERP Kubernetes 部署脚本
set -e

CLUSTER_NAME="cuba-cluster"

echo "🚀 开始 CUBA ERP Kubernetes 部署..."

# 1. 检查工具
command -v kind >/dev/null 2>&1 || { echo "❌ 需要安装 kind"; exit 1; }
command -v kubectl >/dev/null 2>&1 || { echo "❌ 需要安装 kubectl"; exit 1; }
command -v helm >/dev/null 2>&1 || { echo "❌ 需要安装 helm"; exit 1; }
command -v istioctl >/dev/null 2>&1 || { echo "❌ 需要安装 istioctl"; exit 1; }

# 2. 创建集群
if kind get clusters | grep -q "^$CLUSTER_NAME$"; then
    echo "✅ 集群 '$CLUSTER_NAME' 已存在"
else
    echo "📦 正在创建 Kind 集群 '$CLUSTER_NAME'..."
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

# 3. 安装 Istio
echo "🕸️  正在安装 Istio..."
istioctl install --set profile=demo -y

# 4. 创建命名空间并启用注入
echo "💉 正在配置命名空间..."
kubectl apply -f deploy/k8s/infra/namespaces.yaml

# 5. 部署 Secrets
echo "🔐 正在部署 Secrets..."
kubectl apply -f deploy/k8s/infra/secrets.yaml

# 6. 部署 PostgreSQL
echo "🐘 正在部署 PostgreSQL..."
kubectl apply -f deploy/k8s/infra/postgres.yaml

# 等待 PostgreSQL 就绪
echo "⏳ 等待 PostgreSQL 就绪..."
kubectl wait --for=condition=ready pod -l app=cuba-postgres -n cuba-system --timeout=120s

# 7. 构建并加载镜像
# 服务定义: 名称:领域:命名空间
SERVICES=(
    "auth-service:iam:cuba-iam"
    "rbac-service:iam:cuba-iam"
    "oauth-service:iam:cuba-iam"
    "gl-service:fi:cuba-fi"
    "ap-service:fi:cuba-fi"
    "ar-service:fi:cuba-fi"
    "coa-service:fi:cuba-fi"
    "co-service:fi:cuba-fi"
    "tr-service:fi:cuba-fi"
)

echo "🏗️  正在构建并加载镜像..."
for SERVICE_DEF in "${SERVICES[@]}"; do
    IFS=':' read -r SVC DOMAIN NAMESPACE <<< "$SERVICE_DEF"

    DOCKERFILE="apps/$DOMAIN/$SVC/Dockerfile"

    if [ ! -f "$DOCKERFILE" ]; then
        echo "   ⚠️  跳过 $SVC - Dockerfile 未找到: $DOCKERFILE"
        continue
    fi

    IMAGE_NAME="cuba-erp/$SVC:latest"
    echo "   正在构建 $IMAGE_NAME..."
    docker build -t "$IMAGE_NAME" -f "$DOCKERFILE" .

    echo "   正在加载 $IMAGE_NAME 到 Kind..."
    kind load docker-image "$IMAGE_NAME" --name "$CLUSTER_NAME"
done

# 8. 通过 Helm 部署服务
echo "🚀 正在部署微服务..."
for SERVICE_DEF in "${SERVICES[@]}"; do
    IFS=':' read -r SVC DOMAIN NAMESPACE <<< "$SERVICE_DEF"

    VALUES_FILE="deploy/k8s/values/$SVC.yaml"

    if [ ! -f "$VALUES_FILE" ]; then
        echo "   ⚠️  跳过 $SVC - Values 文件未找到: $VALUES_FILE"
        continue
    fi

    echo "   正在安装 $SVC 到 $NAMESPACE..."
    helm upgrade --install "$SVC" deploy/k8s/charts/microservice \
        -f "$VALUES_FILE" \
        --set image.tag=latest \
        --namespace "$NAMESPACE"
done

# 9. 应用 Istio Gateway 和 mTLS
echo "🚪 正在配置 Gateway 和 mTLS..."
kubectl apply -f deploy/k8s/infra/istio-gateway.yaml
kubectl apply -f deploy/k8s/infra/istio-mtls.yaml
kubectl apply -f deploy/k8s/infra/authorization-policies.yaml

echo "🎉 部署完成!"
echo "   运行 'kubectl get pods -A | grep cuba' 查看状态"
echo "   访问服务: http://localhost/api/v1/..."
