# ═══════════════════════════════════════════════════════════════════════════════
# VORTEX Core Engine Dockerfile
# ═══════════════════════════════════════════════════════════════════════════════
#
# ⚠️  DEPLOYMENT STACK: Colima + Minikube + Tilt (NO DOCKER DESKTOP)
#
# This image is designed to be deployed via:
#   1. Colima - Docker runtime (NOT Docker Desktop)
#   2. Minikube - Local Kubernetes cluster (docker driver)
#   3. Tilt - Development workflow orchestration
#
# DO NOT USE DOCKER DESKTOP OR DOCKER COMPOSE DIRECTLY.
# Always run: `tilt up` from the repository root.
#
# Build via Tilt (automatic):
#   $ cd /path/to/vortex && tilt up
#
# Manual build (for debugging only):
#   $ eval $(minikube docker-env)
#   $ docker build -t vortex-core:latest -f docker/core/Dockerfile .
#
# ═══════════════════════════════════════════════════════════════════════════════

# Multi-stage build for minimal final image
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace
COPY Cargo.toml ./
COPY crates ./crates
COPY proto ./proto

# Build release
RUN cargo generate-lockfile && \
    cargo update -p home --precise 0.5.9 && \
    cargo update -p base64ct --precise 1.6.0
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
