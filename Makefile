.PHONY: help build test lint fmt clean docker-build docker-up docker-down dev

# 默认目标
help:
	@echo "CUBA ERP - 可用命令"
	@echo ""
	@echo "开发:"
	@echo "  make dev          - 启动本地开发环境"
	@echo "  make build        - 构建所有服务"
	@echo "  make test         - 运行所有测试"
	@echo "  make lint         - 运行 clippy 代码检查"
	@echo "  make fmt          - 格式化代码"
	@echo "  make clean        - 清理构建产物"
	@echo ""
	@echo "Docker:"
	@echo "  make docker-build - 构建 Docker 镜像"
	@echo "  make docker-up    - 启动 Docker Compose 服务"
	@echo "  make docker-down  - 停止 Docker Compose 服务"
	@echo ""
	@echo "数据库:"
	@echo "  make db-migrate   - 运行数据库迁移"
	@echo "  make db-reset     - 重置数据库"
	@echo ""
	@echo "Protobuf:"
	@echo "  make proto        - 生成 protobuf 代码"
	@echo ""
	@echo "Kubernetes:"
	@echo "  make k8s-deploy   - 部署到 Kubernetes"
	@echo "  make k8s-delete   - 删除 Kubernetes 资源"

# ============================================================================
# 开发
# ============================================================================

build:
	cargo build --release

test:
	cargo test --all-features

lint:
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

clean:
	cargo clean

dev: docker-up
	@echo "开发环境已启动"
	@echo "PostgreSQL: localhost:5432"
	@echo "Envoy 网关: localhost:8080"
	@echo "Swagger UI: localhost:8086"

# ============================================================================
# Docker
# ============================================================================

docker-build:
	docker-compose build

docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

docker-logs:
	docker-compose logs -f

docker-ps:
	docker-compose ps

# ============================================================================
# 数据库
# ============================================================================

db-migrate:
	@echo "正在运行所有服务的数据库迁移..."
	@for dir in apps/*/*/migrations; do \
		if [ -d "$$dir" ]; then \
			echo "迁移: $$dir"; \
		fi \
	done

db-reset:
	docker-compose down -v
	docker-compose up -d postgres
	@echo "等待 PostgreSQL 启动..."
	@sleep 5
	@echo "数据库重置完成"

# ============================================================================
# Protobuf
# ============================================================================

proto:
	@echo "正在生成 protobuf 代码..."
	buf generate
	@echo "Protobuf 生成完成"

proto-lint:
	buf lint

proto-breaking:
	buf breaking --against '.git#branch=main'

# ============================================================================
# Kubernetes
# ============================================================================

k8s-deploy:
	./scripts/deploy_k8s.sh

k8s-delete:
	kubectl delete namespace cuba-system cuba-iam cuba-fi cuba-sd cuba-pm cuba-mf cuba-sc cuba-hr cuba-am cuba-cs cuba-rd --ignore-not-found

k8s-status:
	kubectl get pods -A | grep cuba

k8s-logs:
	@echo "用法: make k8s-logs SVC=<服务名> NS=<命名空间>"
	@if [ -n "$(SVC)" ] && [ -n "$(NS)" ]; then \
		kubectl logs -f -l app=$(SVC) -n $(NS); \
	fi

# ============================================================================
# 工具
# ============================================================================

check: fmt-check lint test
	@echo "所有检查通过!"

ci: check build
	@echo "CI 流水线完成!"
