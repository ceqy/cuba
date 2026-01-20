# Envoy 配置文件说明

## 文件列表

### 1. envoy.yaml (Kubernetes 环境)
- **用途**: Kubernetes 集群部署
- **服务地址**: 使用 K8s Service DNS 名称
  - `auth-service.cuba-iam.svc.cluster.local:50051`
  - `rbac-service.cuba-iam.svc.cluster.local:50052`
  - `gl-service.cuba-fi.svc.cluster.local:50060`
- **不要修改**: 这是生产环境配置

### 2. envoy-docker.yaml (Docker 环境)
- **用途**: 本地 Docker Compose 开发测试
- **服务地址**: 使用 Docker 容器名称
  - `cuba-auth-service:50051`
  - `cuba-rbac-service:50052`
  - `cuba-gl-service:50052`
- **可以修改**: 用于本地开发调试

## 使用方法

### Kubernetes 部署
```bash
kubectl create configmap envoy-config \
  --from-file=envoy.yaml=deploy/envoy/envoy.yaml \
  -n cuba-system
```

### Docker Compose 部署
```yaml
# docker-compose.yaml
services:
  envoy:
    image: envoyproxy/envoy:v1.31-latest
    volumes:
      - ./deploy/envoy/envoy-docker.yaml:/etc/envoy/envoy.yaml:ro
```

## 端口说明

### Auth Service
- 容器内部: `50051`
- K8s Service: `50051`
- Docker 映射: `50051:50051`

### RBAC Service
- 容器内部: `50052`
- K8s Service: `50052`
- Docker 映射: `50052:50052`

### GL Service
- 容器内部: `50052`
- K8s Service: `50060` (通过 Service 映射)
- Docker 映射: `50060:50052`

## 注意事项

1. **不要混用配置文件**
   - Kubernetes 用 `envoy.yaml`
   - Docker 用 `envoy-docker.yaml`

2. **端口配置差异**
   - K8s: 使用 Service 端口
   - Docker: 使用容器内部端口

3. **服务发现**
   - K8s: DNS 自动解析
   - Docker: 依赖 Docker 网络

## 修改历史

### 2026-01-20
- 创建 `envoy-docker.yaml` 用于 Docker 环境
- 保持 `envoy.yaml` 不变,用于 Kubernetes
- 移除不存在的 gRPC 服务引用
