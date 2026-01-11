FROM rust:1.88 AS builder
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY libs ./libs
COPY apps ./apps
COPY protos ./protos
COPY third_party ./third_party
RUN cargo build --release -p pm-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pm-service /usr/local/bin/pm-service
WORKDIR /app
EXPOSE 50076
CMD ["pm-service"]
