# syntax=docker/dockerfile:1
# ─────────────────────────────────────────────────────────────────────────────
# Build from REPO ROOT:
#   docker build --file docker/worker.Dockerfile --tag transcoder/worker:latest .
#
# App:      apps/worker
# Packages: none (self-contained)
# NOTE: tokio-uring is Linux-only — this image MUST be built on Linux.
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
COPY apps/worker/Cargo.toml apps/worker/Cargo.toml

# Minimal workspace — worker only
RUN printf '[workspace]\nmembers = ["apps/worker"]\nresolver = "2"\n' > Cargo.toml

# Stub src so cargo chef prepare can detect crate types
RUN mkdir -p apps/worker/src && echo 'fn main(){}' > apps/worker/src/main.rs

RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 2: Builder ──────────────────────────────────────────────────────────
FROM chef AS builder
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.lock ./
COPY apps/worker/ apps/worker/

RUN printf '[workspace]\nmembers = ["apps/worker"]\nresolver = "2"\n' > Cargo.toml
RUN cargo build --release -p worker

# ── Stage 3: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/worker /usr/local/bin/worker
ENV RUST_LOG=info
CMD ["worker"]
