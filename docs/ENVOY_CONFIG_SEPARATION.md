# Envoy 配置分离完成

## 📋 问题说明

之前我错误地修改了你的 Kubernetes 配置文件 (`deploy/envoy/envoy.yaml`),导致配置来回改动。

## ✅ 解决方案

现在已经创建了**两个独立的配置文件**:

### 1. envoy.yaml - Kubernetes 环境 (已恢复原样)
```yaml
# 用于 Kubernetes 集群部署
# 使用 K8s Service DNS 名称

clusters:
  - name: auth_service
    endpoints:
      - address: auth-service.cuba-iam.svc.cluster.local:50051

  - name: rbac_service
    endpoints:
      - address: rbac-service.cuba-iam.svc.cluster.local:50052

  - name: gl_service
    endpoints:
      - address: gl-service.cuba-fi.svc.cluster.local:50060
```

**状态**: ✅ 已恢复到原始配置,不再修改

### 2. envoy-docker.yaml - Docker 环境 (新建)
```yaml
# 用于本地 Docker Compose 开发
# 使用 Docker 容器名称

clusters:
  - name: auth_service
    endpoints:
      - address: cuba-auth-service:50051

  - name: rbac_service
    endpoints:
      - address: cuba-rbac-service:50052

  - name: gl_service
    endpoints:
      - address: cuba-gl-service:50052
```

**状态**: ✅ 新建文件,用于 Docker 本地测试

## 📁 文件结构

```
deploy/envoy/
├── envoy.yaml          # Kubernetes 配置 (不再修改)
├── envoy-docker.yaml   # Docker 配置 (新建)
├── proto.pb            # gRPC Proto 描述符
└── README.md           # 配置说明文档
```

## 🔧 Docker Compose 配置更新

已更新 `docker-compose.yaml`,使用 Docker 专用配置:

```yaml
services:
  envoy:
    image: envoyproxy/envoy:v1.31-latest
    volumes:
      - ./deploy/envoy/envoy-docker.yaml:/etc/envoy/envoy.yaml:ro  # 使用 Docker 配置
      - ./deploy/envoy/proto.pb:/etc/envoy/proto.pb:ro
```

## 🎯 配置差异说明

| 项目 | Kubernetes (envoy.yaml) | Docker (envoy-docker.yaml) |
|------|------------------------|---------------------------|
| Auth Service | `auth-service.cuba-iam.svc.cluster.local:50051` | `cuba-auth-service:50051` |
| RBAC Service | `rbac-service.cuba-iam.svc.cluster.local:50052` | `cuba-rbac-service:50052` |
| GL Service | `gl-service.cuba-fi.svc.cluster.local:50060` | `cuba-gl-service:50052` |
| 服务发现 | K8s DNS | Docker 网络 |
| 用途 | 生产环境 | 本地开发 |

## ⚠️ 重要说明

### GL Service 端口差异
- **Kubernetes**: 使用 Service 端口 `50060`
  - K8s Service 将 50060 映射到 Pod 的 50052
- **Docker**: 直接使用容器端口 `50052`
  - docker-compose 将容器的 50052 映射到宿主机的 50060

这是**正常的配置差异**,不是错误!

## 📝 使用指南

### Kubernetes 部署
```bash
# 使用原始配置
kubectl create configmap envoy-config \
  --from-file=envoy.yaml=deploy/envoy/envoy.yaml \
  -n cuba-system
```

### Docker 本地开发
```bash
# 自动使用 envoy-docker.yaml
docker-compose up -d envoy
```

## 🔄 重启服务

如果需要应用新配置:

```bash
# Docker 环境
docker-compose restart envoy

# 或者
docker restart cuba-envoy
```

## ✅ 验证

### 检查 Envoy 配置加载
```bash
docker logs cuba-envoy --tail 20
```

### 测试 API 连接
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "demo_user",
    "password": "Demo123456",
    "tenant_id": "default"
  }'
```

## 🎉 总结

### 问题
- ❌ 之前修改了 Kubernetes 配置文件
- ❌ 导致配置来回改动
- ❌ 影响了你的工作

### 解决
- ✅ 创建了独立的 Docker 配置文件
- ✅ 恢复了 Kubernetes 配置到原样
- ✅ 更新了 docker-compose 使用新配置
- ✅ 添加了详细的文档说明

### 承诺
- ✅ **不再修改 `envoy.yaml`**
- ✅ Docker 测试只使用 `envoy-docker.yaml`
- ✅ 两个环境的配置完全独立

## 📚 相关文档

- **配置说明**: `deploy/envoy/README.md`
- **Docker Compose**: `docker-compose.yaml`
- **Swagger UI**: `docs/openapi/TESTING_GUIDE.md`

---

**修改日期**: 2026-01-20
**状态**: ✅ 配置分离完成,不再互相干扰
