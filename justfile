# CUBA Enterprise Monorepo - Justfile
# https://github.com/casey/just
#
# 安装: cargo install just
# 使用: just <recipe>

# 默认显示所有可用命令
default:
    @just --list

# ================================
# 开发环境
# ================================

# 构建所有服务
build:
    cargo build --workspace

# 构建发布版本
build-release:
    cargo build --workspace --release

# 运行所有测试
test:
    cargo test --workspace

# 快速检查编译
check:
    cargo check --workspace

# 格式化代码
fmt:
    cargo fmt --all

# 检查代码格式
fmt-check:
    cargo fmt --all -- --check

# 运行 clippy 检查
lint:
    cargo clippy --workspace -- -D warnings

# 清理构建产物
clean:
    cargo clean

# ================================
# 服务运行
# ================================

# 运行 auth-service
run-auth:
    cargo run -p auth-service

# 运行 sales-service
run-sales:
    cargo run -p sales-service

# ================================
# Docker 基础设施
# ================================

# 启动 Docker 容器
up:
    docker-compose up -d

# 停止 Docker 容器
down:
    docker-compose down

# 查看 Docker 日志
logs:
    docker-compose logs -f

# 重启 Docker 容器
restart:
    docker-compose restart

# 清理 Docker（删除 volumes）
docker-clean:
    docker-compose down -v --remove-orphans

# ================================
# 数据库
# ================================

# 运行数据库迁移
migrate:
    sqlx migrate run

# 创建新迁移 (用法: just migrate-create <name>)
migrate-create name:
    sqlx migrate add {{name}}

# 回滚迁移
migrate-revert:
    sqlx migrate revert

# ================================
# Proto 生成
# ================================

# 生成 proto 代码
proto:
    @echo "Generating proto files..."
    cargo build -p auth-service -p sales-service
    @echo "Proto generation complete."

# ================================
# 一键操作
# ================================

# 初始化项目：启动基础设施 + 迁移
setup: up migrate
    @echo "✅ 项目初始化完成！"
    @echo "运行服务: just run-auth"

# CI 检查：格式 + lint + 测试
ci: fmt-check lint test
    @echo "✅ CI 检查通过！"

# 完整检查：格式化 + lint + 构建 + 测试
all: fmt lint build test
    @echo "✅ 所有检查通过！"

# 快速开发：检查 + 构建
dev: check build
    @echo "✅ 开发构建完成！"

# ================================
# 工具
# ================================

# 更新所有依赖
update:
    cargo update

# 安全审计
audit:
    cargo audit

# 查看依赖树
deps:
    cargo tree

# 查看过期依赖
outdated:
    cargo outdated

# 生成文档
doc:
    cargo doc --workspace --no-deps --open

# ================================
# 版本信息
# ================================

# 显示 Rust 工具链版本
version:
    @echo "Rust: $(rustc --version)"
    @echo "Cargo: $(cargo --version)"
    @echo "Just: $(just --version)"
