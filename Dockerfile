FROM rust:1.83-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Limit parallel jobs to avoid OOM on free-tier build machines
ENV CARGO_BUILD_JOBS=2
RUN cargo build --release -p errand-api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/errand-api /usr/local/bin/errand-api

ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080

CMD ["errand-api"]
