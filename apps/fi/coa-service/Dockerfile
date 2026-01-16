FROM rust:1.88 AS builder

# Install protoc for build.rs
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY libs ./libs
COPY apps ./apps
COPY protos ./protos
COPY third_party ./third_party

# Build the COA service
RUN cargo build --release -p coa-service

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/coa-service /usr/local/bin/coa-service

# Copy migrations
COPY apps/fi/coa-service/migrations /app/migrations

WORKDIR /app

EXPOSE 50060

CMD ["coa-service"]
