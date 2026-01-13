# Cuba ERP Microservice Dockerfile (Multi-stage build)
# Build stage - Use Rust 1.85+ for Edition 2024 support
FROM rust:1.92-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig protobuf-dev

WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY libs/ libs/
COPY protos/ protos/
COPY third_party/ third_party/

# Copy service source
ARG SERVICE_PATH
COPY ${SERVICE_PATH} ${SERVICE_PATH}

# Build release binary
ARG SERVICE_NAME
ENV SQLX_OFFLINE=true
RUN cargo build --release -p ${SERVICE_NAME}

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates libgcc

WORKDIR /app

ARG SERVICE_NAME
COPY --from=builder /app/target/release/${SERVICE_NAME} /app/service

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:9090/health || exit 1

EXPOSE 50051
EXPOSE 9090

ENV RUST_LOG=info
ENV DATABASE_URL=postgres://postgres:postgres@postgres:5432/cuba

CMD ["/app/service"]
