# Database Migrations

此目录用于数据库架构的 **基础设施初始化 (Infrastructure Initialization)**。

## 目录结构

- `000_init_databases.sh`: **Day 0 脚本**。用于在全新的 Postgres 实例中创建所有微服务所需的 40+ 个数据库。通常由 Docker 容器启动时自动调用。

## 关于 Schema Migration (表结构变更)

**注意**：此目录**不包含**具体的业务表结构（Create Table）脚本。

根据微服务最佳实践，每个服务应当**拥有并管理自己的数据 Schema**。具体的表结构迁移脚本位于每个具体的服务代码库中。

例如：
- 总账服务表结构: `apps/finance/gl-service/migrations/*.sql`
- 库存服务表结构: `apps/supplychain/inventory-service/migrations/*.sql`

这些服务级的迁移通常在服务启动时通过 `sqlx migrate run` 或 CI/CD 管道执行。

## 如何使用

### 本地开发 (Docker Compose)
当你运行 `docker-compose up db` 时，Postgres 容器会自动挂载并执行 `000_init_databases.sh`，一次性为你准备好所有服务的空数据库。

### 手动执行
```bash
./migrations/000_init_databases.sh
```
（确保设置了 `POSTGRES_USER` 和 `POSTGRES_DB` 环境变量）
