# 🚀 快速开始指南

欢迎加入 CUBA 企业级微服务项目！本指南将帮助你在 **5 分钟内** 启动并运行整个系统。

## 📋 前置要求

确保你的开发环境已安装以下工具：

- **Rust** (1.70+): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Docker Desktop**: [下载安装](https://www.docker.com/products/docker-desktop)
- **sqlx-cli**: `cargo install sqlx-cli --no-default-features --features postgres`

## ⚡ 一键启动（推荐新人）

```bash
# 1. 克隆项目后进入目录
cd /path/to/cuba

# 2. 启动基础设施（数据库、消息队列等）
docker-compose up -d

# 3. 运行数据库迁移
sqlx migrate run

# 4. 一键启动服务（Auth 后端 + 网关 + Swagger UI）
./scripts/start.sh
```

**完成！** 现在你可以访问：
- **API 网关**: http://localhost:8080
- **Swagger UI**: http://localhost:8081
- **测试账号**: 见 [docs/test_accounts.md](file:///Users/x/x/docs/test_accounts.md)

## 🧪 验证安装

运行以下命令测试登录接口：

```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{
    "tenantId": "T001",
    "username": "admin",
    "password": "Admin123!"
  }'
```

如果返回包含 `access_token` 的 JSON，说明一切正常！

## 📚 进阶操作

### 使用 Makefile（推荐）

```bash
make help           # 查看所有可用命令
make setup          # 一键初始化项目（等同于上面的步骤 2-3）
make run-auth       # 启动 Auth 服务
make test           # 运行所有测试
make fmt            # 格式化代码
```

### 使用 Just（可选）

如果你安装了 `just` (`cargo install just`)：

```bash
just                # 查看所有可用命令
just setup          # 一键初始化
just run-auth       # 启动 Auth 服务
```

## 🛠️ 常见问题

### 1. 端口冲突
如果 8080 端口被占用：
```bash
lsof -i :8080       # 查看占用进程
kill -9 <PID>       # 杀掉进程
```

### 2. 数据库连接失败
确保 Docker 容器正在运行：
```bash
docker ps           # 应该看到 cuba_postgres
```

### 3. CORS 跨域错误
已在 Envoy 配置中启用 CORS，如仍有问题请重启网关：
```bash
docker restart envoy-transcoder
```

## 📖 下一步

- 阅读 [架构文档](file:///Users/x/x/docs/IDENTITY_PLATFORM_ARCHITECTURE.md)
- 查看 [测试账号列表](file:///Users/x/x/docs/test_accounts.md)
- 探索 [Proto 定义](file:///Users/x/x/protos/)

## 💡 提示

- **推荐工具链**: 使用 `Makefile` 而非直接运行脚本，命令更统一。
- **开发模式**: 修改代码后无需重启，`cargo` 会自动重新编译。
- **日志查看**: `docker-compose logs -f` 查看基础设施日志。
