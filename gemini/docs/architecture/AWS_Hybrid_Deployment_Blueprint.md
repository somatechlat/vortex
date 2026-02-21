# VORTEX AWS Hybrid Deployment Blueprint
Document ID: VTX-ARCH-AWS-001  
Version: 1.0.0  
Date: 2026-02-21  
Status: Execution Baseline

## 1. Objective
Define a production-oriented deployment architecture that:
- runs CPU services locally and/or on AWS serverless/container services,
- runs GPU inference on AWS managed GPU endpoints,
- supports real model execution and real image-generation testing.

## 2. Deployment Domains
- Domain A: Local CPU development (`docker compose`)
- Domain B: AWS CPU control plane (`ECS Fargate`, `API Gateway`, `Lambda` where applicable)
- Domain C: AWS GPU inference plane (`SageMaker real-time endpoints`)

## 3. Layer Separation
### 3.1 CPU Control Layer
Responsibilities:
- API ingress and auth
- DAG submission and orchestration
- run state, metadata, audit logs
- queueing and dispatch decisions

Recommended AWS services:
- `API Gateway` for external API entry (serverless)
- `ECS Fargate` for long-running control-plane services
- `Lambda` for short async automation tasks (notifications, webhooks, cleanup jobs)
- `RDS PostgreSQL` for transactional state
- `ElastiCache Redis` for queue/state acceleration

### 3.2 GPU Inference Layer
Responsibilities:
- model loading
- inference execution
- artifact generation

Recommended AWS services:
- `SageMaker real-time endpoint` for diffusion and other GPU-bound workloads
- `S3` for model and output artifacts
- `ECR` for immutable worker images

## 4. Model Portfolio (Testing Baseline)
- Image generation:
  - SDXL base/refiner
  - Flux-dev class models
- Optional extensions:
  - ControlNet variants
  - inpainting/upscaling pipelines
- Validation policy:
  - fixed-seed prompt set for repeatability checks
  - latency and failure-rate measurements per model and instance class

## 5. Service-to-Function Mapping
- User/API request handling: `API Gateway` -> `ECS Fargate` control service
- Workflow scheduling: `ECS Fargate` core runtime
- Artifact storage: `S3`
- GPU inference call: `ECS Fargate` core runtime -> `SageMaker InvokeEndpoint`
- Build pipeline: `CodeBuild` -> `ECR`
- Operational telemetry: `CloudWatch Logs/Metrics` (+ optional X-Ray)

## 6. Runtime Policy
- Local machine policy:
  - CPU-only execution paths and smoke tests.
- AWS policy:
  - real image generation only on GPU real-time endpoint.
  - serverless endpoint used only for API-contract smoke tests.
- Image immutability:
  - SageMaker model image must use ECR digest URI (`@sha256:`).

## 7. Network and Security Baseline
- Private subnets for compute services and data stores.
- Least-privilege IAM for SageMaker execution role.
- Encryption at rest:
  - S3 bucket encryption enabled
  - EFS encryption enabled
- Secrets strategy:
  - secrets in Vault/Secrets Manager; no hardcoded credentials.

## 8. Rollout Plan
1. Stage 0: Local CPU validation (`docker compose`, API smoke, UI checks via `bun`)
2. Stage 1: AWS infra apply + GPU endpoint creation
3. Stage 2: Real model inference tests with stored artifact evidence
4. Stage 3: Load, soak, and failure-injection tests
5. Stage 4: Production readiness review and go-live gate

## 9. Acceptance Criteria
- AC-001 CPU stack runs locally on this machine.
- AC-002 AWS stack deploys with immutable worker image URI.
- AC-003 Real GPU endpoint returns image outputs for approved prompts.
- AC-004 Observability captures request ID, run ID, endpoint latency, and error class.
- AC-005 Deployment docs match actual scripts and Terraform resources.

## 10. Current Repository Alignment
- Terraform infra: `gemini/infra/main.tf`, `gemini/infra/main_serverless.tf`
- Deploy script: `gemini/scripts/deploy_aws.sh`
- Worker runtimes:
  - CPU: `gemini/docker/worker/Dockerfile`
  - GPU: `gemini/docker/Dockerfile.worker.cuda`
