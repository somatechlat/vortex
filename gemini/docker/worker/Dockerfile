FROM python:3.12-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/worker
COPY worker /app/worker

ARG WORKER_REQUIREMENTS=requirements.sandbox.txt
RUN pip install --no-cache-dir -r "${WORKER_REQUIREMENTS}"

ENV PYTHONUNBUFFERED=1
ENV PYTHONPATH=/app/worker
CMD ["python", "-m", "vortex_worker.main"]
