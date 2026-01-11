FROM rust:1.88 AS builder
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY libs ./libs
COPY apps ./apps
COPY protos ./protos
COPY third_party ./third_party
RUN cargo build --release -p tr-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/tr-service /usr/local/bin/tr-service
WORKDIR /app
EXPOSE 50054
CMD ["tr-service"]
