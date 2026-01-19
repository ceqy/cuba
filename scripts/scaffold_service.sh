#!/bin/bash
set -e

SERVICE_NAME=$1
PROTO_DIR=$2 # e.g. "fi/ap"
SERVICE_PORT=$3
DB_NAME=$4

if [ -z "$SERVICE_NAME" ] || [ -z "$PROTO_DIR" ] || [ -z "$SERVICE_PORT" ] || [ -z "$DB_NAME" ]; then
  echo "Usage: ./scaffold_service.sh <service-name> <proto-dir> <port> <db-name>"
  echo "Example: ./scaffold_service.sh ap-service fi/ap 50053 cuba_fi_ap"
  exit 1
fi

# Extract domain (first part of proto dir) for simpler app structure
# e.g. fi/ap -> fi
DOMAIN=$(echo "$PROTO_DIR" | cut -d'/' -f1)

# Target: apps/fi/ap-service
BASE_DIR="apps/$DOMAIN/$SERVICE_NAME"
mkdir -p "$BASE_DIR/src/api" "$BASE_DIR/src/application" "$BASE_DIR/src/domain" "$BASE_DIR/src/infrastructure" "$BASE_DIR/migrations"

# Create mod.rs files
touch "$BASE_DIR/src/api/mod.rs" \
      "$BASE_DIR/src/application/mod.rs" \
      "$BASE_DIR/src/domain/mod.rs" \
      "$BASE_DIR/src/infrastructure/mod.rs"

echo "Scaffolding $SERVICE_NAME in $BASE_DIR..."

# 1. Cargo.toml (Leveraging Workspace Dependencies with dot notation)
cat > "$BASE_DIR/Cargo.toml" <<EOF
[package]
name = "$SERVICE_NAME"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
# Shared Infrastructure
cuba-core.workspace = true
cuba-errors.workspace = true
cuba-cqrs.workspace = true
cuba-database.workspace = true
cuba-messaging.workspace = true
cuba-telemetry.workspace = true
cuba-service = { path = "../../../libs/cuba-service" }
anyhow.workspace = true

# Async
tokio.workspace = true
async-trait.workspace = true

# gRPC
tonic.workspace = true
tonic-prost.workspace = true
tonic-reflection.workspace = true
prost.workspace = true
prost-types.workspace = true

# Database
sqlx.workspace = true

# Serialization
serde.workspace = true
serde_json.workspace = true

# Utilities
uuid.workspace = true
chrono.workspace = true
rust_decimal.workspace = true
thiserror.workspace = true
tracing.workspace = true
dotenvy.workspace = true

[build-dependencies]
tonic-build.workspace = true
tonic-prost-build.workspace = true
EOF

# 2. build.rs
# Find the first .proto file in the dir to guess the name (simplified)
PROTO_FILE=$(find protos/$PROTO_DIR -name "*.proto" | head -n 1)
# Calculate relative path from build.rs to proto
REL_PROTO_PATH="../../../$PROTO_FILE"
REL_COMMON_PATH="../../../protos/common/common.proto"
REL_INCLUDE_PATH="../../../protos"
REL_THIRD_PARTY="../../../third_party"

cat > "$BASE_DIR/build.rs" <<EOF
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    // Generate File Descriptor Set for Reflection & Envoy
    tonic_prost_build::configure()
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("descriptor.bin"))
        .compile_protos(
            &["$REL_PROTO_PATH", "$REL_COMMON_PATH"],
            &["$REL_INCLUDE_PATH", "$REL_THIRD_PARTY"],
        )?;
    Ok(())
}
EOF

# 3. src/lib.rs
cat > "$BASE_DIR/src/lib.rs" <<EOF
pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;
EOF

# 4. src/main.rs
cat > "$BASE_DIR/src/main.rs" <<EOF
use tonic::transport::Server;
use cuba_database::{DatabaseConfig, init_pool};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Init Telemetry
    cuba_telemetry::init_telemetry();

    // 2. Load Config
    let addr = "0.0.0.0:$SERVICE_PORT".parse()?;
    info!("Starting $SERVICE_NAME on {}", addr);

    // 3. Init Database
    let db_config = DatabaseConfig::default();
    let _pool = init_pool(&db_config).await?;

    // 4. Init Reflection
    let descriptor = include_bytes!(concat!(env!("OUT_DIR"), "/descriptor.bin"));
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(descriptor)
        .build_v1()?;

    info!("Service listening on {}", addr);

    // 5. Start Server
    Server::builder()
        .add_service(reflection_service)
        // .add_service(YourGrpcServiceServer::new(YourServiceImpl))
        .serve(addr)
        .await?;

    Ok(())
}
EOF

# 5. Dockerfile (Alpine-based with health check)
cat > "$BASE_DIR/Dockerfile" <<EOF
FROM rust:1.92-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev protobuf-dev pkgconfig

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY libs/ libs/
COPY apps/ apps/
COPY protos/ protos/
COPY third_party/ third_party/

# Build service
ENV SQLX_OFFLINE=true
RUN cargo build --release -p $SERVICE_NAME

# Runtime stage
FROM alpine:3.21

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/$SERVICE_NAME /app/service

# Copy migrations
COPY apps/$DOMAIN/$SERVICE_NAME/migrations /app/migrations

# Environment
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \\
    CMD /app/service --health-check || exit 1

# Expose ports
EXPOSE 50051
EXPOSE 9090

CMD ["/app/service"]
EOF

echo "Scaffold Complete for $SERVICE_NAME"
echo "Note: $BASE_DIR should be automatically picked up by workspace members = [\"apps/*/*\"]"
