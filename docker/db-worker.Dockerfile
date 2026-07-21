# syntax=docker/dockerfile:1
# ─────────────────────────────────────────────────────────────────────────────
# Build from REPO ROOT:
#   docker build --file docker/db-worker.Dockerfile --tag transcoder/db-worker:latest .
#
# App:      apps/db-worker
# Packages: packages/db
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
COPY apps/db-worker/Cargo.toml apps/db-worker/Cargo.toml
COPY packages/db/Cargo.toml    packages/db/Cargo.toml

# Minimal workspace — only what db-worker needs
RUN printf '[workspace]\nmembers = ["apps/db-worker", "packages/db"]\nresolver = "2"\n' > Cargo.toml

# Stub src so cargo chef prepare can detect crate types
RUN mkdir -p apps/db-worker/src         && echo 'fn main(){}' > apps/db-worker/src/main.rs && \
    mkdir -p packages/db/src/bin        && touch packages/db/src/lib.rs packages/db/src/bin/migrate.rs

RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 2: Builder ──────────────────────────────────────────────────────────
FROM chef AS builder
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.lock ./
COPY apps/db-worker/ apps/db-worker/
COPY packages/db/    packages/db/

RUN printf '[workspace]\nmembers = ["apps/db-worker", "packages/db"]\nresolver = "2"\n' > Cargo.toml
RUN cargo build --release -p db-worker

# ── Stage 3: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 libpq5 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/db-worker /usr/local/bin/db-worker
ENV RUST_LOG=info
CMD ["db-worker"]
