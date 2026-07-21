# syntax=docker/dockerfile:1
# ─────────────────────────────────────────────────────────────────────────────
# Migration init-container — runs Diesel migrations then exits (code 0).
#
# Build from REPO ROOT:
#   docker build --file docker/db-migrate.Dockerfile --tag transcoder/db-migrate:latest .
#
# App:      none (binary lives in packages/db)
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
COPY packages/db/Cargo.toml packages/db/Cargo.toml

# Minimal workspace — packages/db only
RUN printf '[workspace]\nmembers = ["packages/db"]\nresolver = "2"\n' > Cargo.toml

# Stub src so cargo chef prepare can detect crate types
RUN mkdir -p packages/db/src/bin && touch packages/db/src/lib.rs packages/db/src/bin/migrate.rs

RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 2: Builder ──────────────────────────────────────────────────────────
FROM chef AS builder
WORKDIR /app

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.lock ./
COPY packages/db/ packages/db/

RUN printf '[workspace]\nmembers = ["packages/db"]\nresolver = "2"\n' > Cargo.toml
RUN cargo build --release --bin migrate

# ── Stage 3: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libpq5 && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/migrate /usr/local/bin/migrate
# Exits 0 on success, non-zero on failure — docker-compose depends_on uses this
CMD ["migrate"]
