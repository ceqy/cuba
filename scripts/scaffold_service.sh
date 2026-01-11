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
mkdir -p "$BASE_DIR/src/api" "$BASE_DIR/src/application" "$BASE_DIR/src/domain" "$BASE_DIR/src/infrastructure"

# Create mod.rs files
touch "$BASE_DIR/src/api/mod.rs" \
      "$BASE_DIR/src/application/mod.rs" \
      "$BASE_DIR/src/domain/mod.rs" \
      "$BASE_DIR/src/infrastructure/mod.rs"

echo "Scaffolding $SERVICE_NAME in $BASE_DIR..."

# 1. Cargo.toml (Leveraging Workspace Dependencies)
cat > "$BASE_DIR/Cargo.toml" <<EOF
[package]
name = "$SERVICE_NAME"
version = "0.1.0"
edition.workspace = true

[dependencies]
# Internal
cuba-core = { workspace = true }
cuba-database = { workspace = true }
cuba-telemetry = { workspace = true }
cuba-cqrs = { workspace = true }

# External
tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }
tonic = { workspace = true }
prost = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
sqlx = { workspace = true }
dotenvy = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
rust_decimal = { workspace = true }
tonic-reflection = { workspace = true }
thiserror = { workspace = true }

[build-dependencies]
tonic-build = { workspace = true }
tonic-prost-build = { workspace = true }
EOF

# 2. build.rs
# Find the first .proto file in the dir to guess the name (simplified)
PROTO_FILE=$(find protos/$PROTO_DIR -name "*.proto" | head -n 1)
# Calculate relative path from build.rs to proto
# apps/fi/ap-service -> ../../../../../protos/fi/ap/ap.proto
# We used DOMAIN, so apps/$DOMAIN/$SERVICE_NAME.
# apps/fi/ap-service -> 3 levels deep.
# So relative path should start with ../../../
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
# We use a compact main utilizing the libraries
cat > "$BASE_DIR/src/main.rs" <<EOF
use tonic::transport::Server;
use cuba_database::{DatabaseConfig, init_pool};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Init Telemetry
    cuba_telemetry::init_telemetry();
    
    // 2. Load Config
    // In a real app we might load strictly typed config, here we assume env vars.
    let addr = "0.0.0.0:$SERVICE_PORT".parse()?;
    info!("Starting $SERVICE_NAME on {}", addr);

    // 3. Init Database
    let db_config = DatabaseConfig::default();
    let _pool = init_pool(&db_config).await?; // Pool is ready, typically passed to repositories

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

# 5. Dockerfile
cat > "$BASE_DIR/Dockerfile" <<EOF
FROM rust:1.88 AS builder
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY libs ./libs
COPY apps ./apps
COPY protos ./protos
COPY third_party ./third_party
RUN cargo build --release -p $SERVICE_NAME

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/$SERVICE_NAME /usr/local/bin/$SERVICE_NAME
WORKDIR /app
EXPOSE $SERVICE_PORT
CMD ["$SERVICE_NAME"]
EOF

echo "Scaffold Complete for $SERVICE_NAME"
echo "Note: $BASE_DIR should be automatically picked up by workspace members = [\"apps/*/*\"]"
