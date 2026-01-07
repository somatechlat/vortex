# VORTEX Python Worker Dockerfile - Production Grade
# Build ARG controls which requirements to use (sandbox/live)
FROM python:3.11-slim-bookworm AS base

ARG VORTEX_MODE=sandbox

WORKDIR /app

# Install system dependencies for posix_ipc and build tools
RUN apt-get update && apt-get install -y \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# ═══════════════════════════════════════════════════════════════
#                    DEPENDENCIES
# ═══════════════════════════════════════════════════════════════
FROM base AS deps

ARG VORTEX_MODE=sandbox

COPY worker/pyproject.toml ./
COPY worker/requirements.sandbox.txt ./requirements.sandbox.txt
COPY worker/requirements.live.txt ./requirements.live.txt

# Install Python dependencies based on mode
RUN pip install --no-cache-dir -r requirements.${VORTEX_MODE}.txt

# ═══════════════════════════════════════════════════════════════
#                    RUNTIME
# ═══════════════════════════════════════════════════════════════
FROM deps AS runtime

# Copy application code
COPY worker/vortex_worker ./vortex_worker

# Create non-root user for security
RUN useradd -m -u 1000 worker
USER worker

# Environment
ENV PYTHONUNBUFFERED=1
ENV PYTHONDONTWRITEBYTECODE=1
ENV VORTEX_SLOT_ID=0
ENV LOG_LEVEL=INFO

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD python -c "import sys; sys.exit(0)"

# Entry point
ENTRYPOINT ["python", "-m", "vortex_worker"]
