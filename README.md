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

## Getting Started

1.  **Start Infrastructure**: `docker-compose up -d`
2.  **Run Migrations**: `sqlx migrate run`
3.  **Build All Services**: `cargo build --workspace`
4.  **Run a Specific Service**: `cargo run -p auth-service`
