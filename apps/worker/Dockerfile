FROM rust:1.80-slim-bookworm AS builder
WORKDIR /usr/src/worker
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/worker/target/release/worker /usr/local/bin/worker
ENV RUST_LOG=info
CMD ["worker"]
