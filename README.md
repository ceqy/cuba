# CUBA Enterprise Monorepo

This repository contains the source code for the **Corporate Unified Business Architecture (CUBA)**, a suite of 40 microservices built with Rust, following DDD, CQRS, and Event Sourcing principles.

## Directory Structure

```
/cuba-monorepo
├── apps/                   # Individual microservice applications
│   ├── auth-service/
│   ├── sales-service/
│   └── ... (38 more services)
│
├── libs/                   # Shared libraries (crates)
│   ├── cuba-core/          # Core DDD/CQRS traits, Event Sourcing logic
│   ├── cuba-config/        # Configuration loading
│   ├── cuba-errors/        # Shared error types
│   ├── cuba-database/      # Database connection and repository helpers
│   ├── cuba-messaging/     # Kafka integration helpers
│   └── cuba-telemetry/     # Tracing and metrics setup
│
├── protos/                 # All gRPC .proto files for all services
│   ├── auth/
│   ├── sales/
│   └── ... (and so on for all 9 business domains)
│
├── config/                 # Service configuration files (YAML)
├── migrations/             # Database migration scripts (sqlx-cli)
├── scripts/                # Helper scripts (e.g., code generation)
├── docker-compose.yml      # Local development environment
└── Cargo.toml              # Workspace root
```

## 🚀 快速开始

**新人推荐**: 请查看 **[快速开始指南 (QUICKSTART.md)](file:///Users/x/x/docs/QUICKSTART.md)** 获取详细的一键启动步骤。

### 极简版（5 分钟）

```bash
docker-compose up -d          # 启动基础设施
sqlx migrate run              # 数据库迁移
./scripts/start.sh            # 一键启动服务（Auth + 网关 + Swagger）
```

访问 http://localhost:8081 查看 Swagger UI。

### 使用 Makefile（推荐）

```bash
make setup      # 初始化项目
make run-auth   # 启动服务
make help       # 查看所有命令
```

## 📚 文档

- [快速开始指南](file:///Users/x/x/docs/QUICKSTART.md) - 新人必读
- [测试账号](file:///Users/x/x/docs/test_accounts.md) - 前端测试凭据
- [架构文档](file:///Users/x/x/docs/IDENTITY_PLATFORM_ARCHITECTURE.md)
