# syntax=docker/dockerfile:1
# ─────────────────────────────────────────────────────────────────────────────
# Build from REPO ROOT:
#   docker build --file docker/tus-server.Dockerfile --tag transcoder/tus-server:latest .
#
# App:      apps/tus-server
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
COPY apps/tus-server/Cargo.toml        apps/tus-server/Cargo.toml
COPY packages/db/Cargo.toml            packages/db/Cargo.toml
COPY packages/redis-conn/Cargo.toml    packages/redis-conn/Cargo.toml
COPY packages/rabbitmq-conn/Cargo.toml packages/rabbitmq-conn/Cargo.toml

# Minimal workspace — only what tus-server needs
RUN printf '[workspace]\nmembers = ["apps/tus-server", "packages/db", "packages/redis-conn", "packages/rabbitmq-conn"]\nresolver = "2"\n' > Cargo.toml

# Stub src so cargo chef prepare can detect crate types
RUN mkdir -p apps/tus-server/src        && echo 'fn main(){}' > apps/tus-server/src/main.rs  && \
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
COPY apps/tus-server/          apps/tus-server/
COPY packages/db/              packages/db/
COPY packages/redis-conn/      packages/redis-conn/
COPY packages/rabbitmq-conn/   packages/rabbitmq-conn/

RUN printf '[workspace]\nmembers = ["apps/tus-server", "packages/db", "packages/redis-conn", "packages/rabbitmq-conn"]\nresolver = "2"\n' > Cargo.toml
RUN cargo build --release -p tus-server

# ── Stage 3: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 libpq5 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/tus-server /usr/local/bin/tus-server
ENV RUST_LOG=info
ENV TUS_PORT=1081
EXPOSE 1081
CMD ["tus-server"]
