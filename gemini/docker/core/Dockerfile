# VORTEX Core Engine Dockerfile
# Multi-stage build for minimal final image
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

# Build release
RUN cargo build --release --package vortex-core

# ═══════════════════════════════════════════════════════════════
#                    RUNTIME IMAGE
# ═══════════════════════════════════════════════════════════════
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/vortex-core /app/vortex-core

# Create non-root user
RUN useradd -m -u 1000 vortex
USER vortex

# Environment
ENV RUST_LOG=info
ENV VORTEX_HTTP_PORT=11188
ENV VORTEX_WS_PORT=11189
ENV VORTEX_METRICS_PORT=11191

# Expose ports (Port Authority: 11000-11999)
EXPOSE 11188 11189 11191

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s \
    CMD curl -f http://localhost:11188/health || exit 1

ENTRYPOINT ["/app/vortex-core"]
