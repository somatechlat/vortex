# VORTEX Deployment Guide

This guide matches the current codebase and supports two deployment tracks:

1. Local CPU stack for development and functional testing
2. AWS test stack for real GPU image generation, with serverless where practical

## Rules

- Use `bun` for all UI package management and scripts.
- Do not use `npm` in this repository.
- Keep deployment ports in the `11000-11999` range.

## Track A: Local CPU Deployment

Path: `gemini/infra/docker/standalone/`

### What it runs

- `vortex-core` (Rust API/runtime)
- `vortex-worker` (Python worker on CPU)
- `vortex-ui`
- `postgres`, `vault`, `keycloak`, `milvus`, `spicedb`

### Why CPU here

`gemini/docker/worker/Dockerfile` defaults to `requirements.sandbox.txt`, which installs CPU PyTorch wheels.

### Start

```bash
cd gemini/infra/docker/standalone
docker compose up -d --build
```

### Validate

```bash
curl -fsS http://localhost:11188/health
curl -fsS http://localhost:11188/info
open http://localhost:11100
```

## Track B: AWS Test Deployment (GPU + Serverless Control Plane)

Path: `gemini/infra/`

### Architecture (serverless when possible)

- Serverless/control plane:
  - ECR for image registry (immutable tags + digest deployment)
  - CodeBuild for container builds
  - ECS/Fargate cluster primitives for CPU services and API expansion
- GPU inference plane:
  - SageMaker real-time endpoint on GPU instance (`ml.g5.xlarge` default)
- Optional serverless test endpoint:
  - SageMaker serverless endpoint for API-contract smoke tests only (not for diffusion load)
- Shared infra:
  - VPC, private/public subnets, NAT
  - S3 model bucket
  - EFS model cache

### Provision and deploy

```bash
./gemini/scripts/deploy_aws.sh <aws_account_id> [aws_region] [gpu_instance_type] [enable_serverless_smoke] [image_tag]
```

Examples:

```bash
./gemini/scripts/deploy_aws.sh 123456789012 us-east-1 ml.g5.xlarge false
./gemini/scripts/deploy_aws.sh 123456789012 us-east-1 ml.g5.2xlarge true v1.0.0-rc1
```

### Terraform variables used

- `account_id`
- `aws_region`
- `sagemaker_gpu_instance_type`
- `sagemaker_initial_instance_count`
- `enable_sagemaker_serverless_smoke`
- `sagemaker_serverless_max_concurrency`
- `sagemaker_serverless_memory_mb`
- `sagemaker_worker_image_uri` (digest URI passed automatically by deploy script)

### Outputs to check

- `sagemaker_endpoint_name`
- `sagemaker_serverless_smoke_endpoint_name` (nullable)
- `ecr_worker_repository_url`

## Real Image Generation Test Flow

1. Deploy GPU endpoint via `deploy_aws.sh`.
2. Invoke the SageMaker endpoint with a real diffusion payload.
3. Assert image bytes are returned and persist output for regression.
4. Repeat with fixed seed to compare deterministic behavior on same hardware class.

## UI / Frontend commands (Bun only)

From `gemini/ui/`:

```bash
bun install
bun run check
bun run dev
```

## Operational Notes

- SageMaker serverless is included for lightweight endpoint/API checks only.
- Real diffusion benchmarking and latency validation must run on GPU real-time endpoints.
- Terraform now enforces immutable image URIs (`@sha256:<digest>`) for SageMaker model deployment.
- Keep Dockerfile split:
  - Local CPU: `gemini/docker/worker/Dockerfile`
  - AWS GPU: `gemini/docker/Dockerfile.worker.cuda`
