# VORTEX Core Engine - Production Image
FROM rust:1.75-slim AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build
RUN cargo build --release --bin vortex-core

# Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/vortex-core /usr/local/bin/vortex-core

# Run as non-root
RUN useradd -m -u 1000 vortex
USER vortex

EXPOSE 11188 11189 11191

CMD ["vortex-core"]