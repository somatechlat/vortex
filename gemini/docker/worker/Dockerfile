# VORTEX Python Worker Dockerfile
# CUDA-enabled for GPU inference
FROM python:3.11-slim-bookworm AS base

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# ═══════════════════════════════════════════════════════════════
#                    DEPENDENCIES
# ═══════════════════════════════════════════════════════════════
FROM base AS deps

COPY worker/pyproject.toml worker/requirements.txt ./

# Install Python dependencies
RUN pip install --no-cache-dir -r requirements.txt

# ═══════════════════════════════════════════════════════════════
#                    RUNTIME
# ═══════════════════════════════════════════════════════════════
FROM deps AS runtime

# Copy application code
COPY worker/vortex_worker ./vortex_worker

# Create non-root user
RUN useradd -m -u 1000 worker
USER worker

# Environment
ENV PYTHONUNBUFFERED=1
ENV PYTHONDONTWRITEBYTECODE=1
ENV VORTEX_SLOT_ID=0

# Entry point
ENTRYPOINT ["python", "-m", "vortex_worker"]
