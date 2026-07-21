# syntax=docker/dockerfile:1
# ─────────────────────────────────────────────────────────────────────────────
# Build from REPO ROOT:
#   docker build --file docker/transcoder.Dockerfile --tag transcoder/transcoder:latest .
#
# App:      apps/transcoder
# Packages: packages/redis-conn
# ─────────────────────────────────────────────────────────────────────────────

FROM rust:bookworm AS chef
RUN apt-get update && apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app

# ── Stage 1: Planner ─────────────────────────────────────────────────────────
FROM chef AS planner
WORKDIR /app

COPY Cargo.lock ./
COPY apps/transcoder/Cargo.toml     apps/transcoder/Cargo.toml
COPY packages/redis-conn/Cargo.toml packages/redis-conn/Cargo.toml

# Minimal workspace — only what transcoder needs
RUN printf '[workspace]\nmembers = ["apps/transcoder", "packages/redis-conn"]\nresolver = "2"\n' > Cargo.toml

# Stub src so cargo chef prepare can detect crate types
RUN mkdir -p apps/transcoder/src     && echo 'fn main(){}' > apps/transcoder/src/main.rs && \
    mkdir -p packages/redis-conn/src && touch packages/redis-conn/src/lib.rs

RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 2: Builder ──────────────────────────────────────────────────────────
FROM chef AS builder
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.lock ./
COPY apps/transcoder/        apps/transcoder/
COPY packages/redis-conn/    packages/redis-conn/

RUN printf '[workspace]\nmembers = ["apps/transcoder", "packages/redis-conn"]\nresolver = "2"\n' > Cargo.toml
RUN cargo build --release -p transcoder

# ── Stage 3: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/transcoder /usr/local/bin/transcoder
ENV RUST_LOG=info
CMD ["transcoder"]
