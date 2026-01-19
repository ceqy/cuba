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
RUN cargo build --release -p ah-service

# Runtime stage
FROM alpine:3.21

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/ah-service /app/service

# Copy migrations
COPY apps/am/ah-service/migrations /app/migrations

# Environment
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD /app/service --health-check || exit 1

# Expose ports
EXPOSE 50051
EXPOSE 9090

CMD ["/app/service"]
