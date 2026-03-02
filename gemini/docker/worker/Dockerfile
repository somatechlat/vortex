# VORTEX Worker - CPU Image
FROM python:3.11-slim

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY worker/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY worker/ .

# Create non-root user
RUN useradd -m -u 1000 vortex
RUN chown -R vortex:vortex /app
USER vortex

CMD ["python", "-m", "vortex_worker"]