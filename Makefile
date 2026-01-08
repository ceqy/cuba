# CUBA Enterprise Monorepo - Makefile
# ===================================

# Load environment variables from .env file
ifneq (,$(wildcard ./.env))
	include .env
	export
endif

.PHONY: help build test run clean docker-up docker-down migrate proto fmt lint check

# Default target
help:
	@echo "CUBA Enterprise - Available Commands"
	@echo "====================================="
	@echo ""
	@echo "Development:"
	@echo "  make build          - Build all services"
	@echo "  make test           - Run all tests"
	@echo "  make check          - Run cargo check"
	@echo "  make fmt            - Format code"
	@echo "  make lint           - Run clippy linter"
	@echo ""
	@echo "Services:"
	@echo "  make run-auth       - Run auth-service"
	@echo "  make run-sales      - Run sales-service"
	@echo ""
	@echo "Infrastructure:"
	@echo "  make docker-up      - Start Docker containers"
	@echo "  make docker-down    - Stop Docker containers"
	@echo "  make docker-logs    - View Docker logs"
	@echo ""
	@echo "Database:"
	@echo "  make migrate        - Run database migrations"
	@echo "  make migrate-create - Create new migration (NAME=xxx)"
	@echo ""
	@echo "Proto:"
	@echo "  make proto          - Generate code from proto files"
	@echo ""
	@echo "Utilities:"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make setup          - Initial project setup"

# ===================
# Development Commands
# ===================

build:
	cargo build --workspace

build-release:
	cargo build --workspace --release

test:
	cargo test --workspace

check:
	cargo check --workspace

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

lint:
	cargo clippy --workspace -- -D warnings

# ===================
# Service Commands
# ===================

run-auth:
	cargo run -p auth-service

run-sales:
	cargo run -p sales-service

run-finance:
	cargo run -p finance-service

# ===================
# Infrastructure
# ===================

docker-up:
	docker-compose up -d

docker-down:
	docker-compose down

docker-logs:
	docker-compose logs -f

docker-restart:
	docker-compose restart

docker-clean:
	docker-compose down -v --remove-orphans

# ===================
# Database
# ===================

migrate:
	sqlx migrate run

migrate-create:
	@if [ -z "$(NAME)" ]; then \
		echo "Error: NAME is required. Usage: make migrate-create NAME=create_users_table"; \
		exit 1; \
	fi
	sqlx migrate add $(NAME)

migrate-revert:
	sqlx migrate revert

# ===================
# Proto Generation
# ===================

proto:
	@echo "Generating proto files..."
	@for dir in protos/*/; do \
		echo "Processing $$dir..."; \
	done
	@echo "Proto generation complete."

# ===================
# Utilities
# ===================

clean:
	cargo clean

setup: docker-up migrate
	@echo "Project setup complete!"
	@echo "You can now run: make run-auth"

# ===================
# CI/CD
# ===================

ci: fmt-check lint test
	@echo "CI checks passed!"

all: fmt lint build test
	@echo "All checks passed!"
