# syntax=docker/dockerfile:1
# ─────────────────────────────────────────────────────────────────────────────
# Build from REPO ROOT:
#   docker build --file docker/http-server.Dockerfile --tag transcoder/http-server:latest .
#
# App:      apps/http-server
# Packages: packages/db, packages/redis-conn, packages/rabbitmq-conn
# ─────────────────────────────────────────────────────────────────────────────

FROM rust:bookworm AS chef
RUN apt-get update && apt-get install -y pkg-config libpq-dev libssl-dev && \
    rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app

# ── Stage 1: Planner ─────────────────────────────────────────────────────────
FROM chef AS planner
WORKDIR /app

COPY Cargo.lock ./
COPY apps/http-server/Cargo.toml       apps/http-server/Cargo.toml
COPY packages/db/Cargo.toml            packages/db/Cargo.toml
COPY packages/redis-conn/Cargo.toml    packages/redis-conn/Cargo.toml
COPY packages/rabbitmq-conn/Cargo.toml packages/rabbitmq-conn/Cargo.toml

# Minimal workspace — only what http-server needs
RUN printf '[workspace]\nmembers = ["apps/http-server", "packages/db", "packages/redis-conn", "packages/rabbitmq-conn"]\nresolver = "2"\n' > Cargo.toml

# Stub src so cargo chef prepare can detect crate types
RUN mkdir -p apps/http-server/src       && echo 'fn main(){}' > apps/http-server/src/main.rs && \
    mkdir -p packages/db/src/bin        && touch packages/db/src/lib.rs packages/db/src/bin/migrate.rs && \
    mkdir -p packages/redis-conn/src    && touch packages/redis-conn/src/lib.rs    && \
    mkdir -p packages/rabbitmq-conn/src && touch packages/rabbitmq-conn/src/lib.rs

RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 2: Builder ──────────────────────────────────────────────────────────
FROM chef AS builder
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.lock ./
COPY apps/http-server/          apps/http-server/
COPY packages/db/               packages/db/
COPY packages/redis-conn/       packages/redis-conn/
COPY packages/rabbitmq-conn/    packages/rabbitmq-conn/

RUN printf '[workspace]\nmembers = ["apps/http-server", "packages/db", "packages/redis-conn", "packages/rabbitmq-conn"]\nresolver = "2"\n' > Cargo.toml
RUN cargo build --release -p http-server

# ── Stage 3: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 libpq5 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/http-server /usr/local/bin/http-server
ENV RUST_LOG=info
EXPOSE 3000
CMD ["http-server"]
