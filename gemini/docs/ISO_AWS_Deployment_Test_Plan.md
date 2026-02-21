# ISO Deployment Test Plan: Local CPU + AWS GPU

## 1. Document Control

- Document ID: VTX-ISO-DEPLOY-TEST-001
- Version: 1.0.0
- Date: 2026-02-21
- System: VORTEX
- Source of truth: repository code in `gemini/`

## 2. Objective

Define a deployable and testable architecture that:

- runs locally on CPU for development
- runs on AWS GPU for real diffusion/image-generation tests
- uses serverless components where they fit operationally

## 3. Scope

In scope:

- Docker CPU local stack
- AWS Terraform stack in `gemini/infra/`
- GPU worker image build and publish
- SageMaker endpoint deployment and invocation testing
- documentation alignment with implemented code

Out of scope:

- production multi-region failover
- cost optimization beyond baseline instance choice
- enterprise IAM least-privilege hardening final pass

## 4. Requirements and Traceability

- R-001 Local CPU stack MUST run with Docker Compose.
- R-002 AWS stack MUST deploy a real-time GPU SageMaker endpoint.
- R-003 Optional serverless endpoint MUST be available for smoke/API tests.
- R-004 Worker image build MUST publish to ECR.
- R-005 UI package workflow MUST use Bun (no npm).
- R-006 Deployment docs MUST reflect implemented scripts and Terraform.

Traceability:

- R-001 -> `gemini/infra/docker/standalone/docker-compose.yml`
- R-002 -> `gemini/infra/main_serverless.tf`
- R-003 -> `gemini/infra/main_serverless.tf`
- R-004 -> `gemini/scripts/deploy_aws.sh`, `gemini/docker/Dockerfile.worker.cuda`
- R-005 -> `gemini/docs/coding_rules.md`, `gemini/ui/`
- R-006 -> `gemini/docs/DEPLOYMENT.md`

## 5. Target Architecture

- Local:
  - `vortex-core` + `vortex-worker` CPU + `vortex-ui` + dependencies via Compose
- AWS CPU control plane (serverless-preferred):
  - ECR, CodeBuild, ECS/Fargate service layer, S3
- AWS inference plane:
  - SageMaker real-time GPU endpoint (`ml.g5.xlarge` default)
- Optional smoke plane:
  - SageMaker serverless endpoint for contract checks

Deployment mode split:
- Mode 1 Local CPU: Full stack on Docker Compose, no GPU requirement.
- Mode 2 AWS hybrid test: CPU/API services on Fargate, image generation on SageMaker GPU endpoint.
- Mode 3 AWS smoke only: CPU/API services + SageMaker serverless smoke endpoint for contract validation.

## 6. Deployment Procedure

1. Build and push GPU worker image to ECR.
2. Resolve ECR digest and pass immutable image URI into Terraform.
3. Apply Terraform with account, region, and endpoint options.
4. Verify outputs for endpoint names and repository URL.
5. Execute real endpoint invocation with real diffusion model payload.
6. Capture latency and output image artifacts.

## 7. Verification and Acceptance Criteria

- AC-001 `docker compose up -d --build` succeeds for local stack.
- AC-002 `terraform apply` succeeds without variable mismatch.
- AC-003 `sagemaker_endpoint_name` is non-empty.
- AC-004 Endpoint invocation returns image output for real model request.
- AC-005 `bun run check` executes successfully for UI.
- AC-006 SageMaker model image is pinned by digest (`@sha256:`), not mutable tag.

## 8. Risks and Mitigations

- GPU cold starts and endpoint warm-up latency.
  - Mitigation: keep initial instance count >= 1 for test windows.
- SageMaker serverless memory limits for diffusion payloads.
  - Mitigation: treat serverless as smoke-only path.
- Dependency drift between CPU and GPU worker environments.
  - Mitigation: pin requirements files and validate both build paths.

## 9. Execution Tasks

1. Provision local CPU stack and validate health endpoints.
2. Provision AWS stack with GPU endpoint.
3. Run real model image generation test and persist artifacts.
4. Run repeatability test with fixed seed on same endpoint class.
5. Record metrics, defects, and remediation actions in task board.

## 10. Records

- Terraform outputs from `gemini/infra/`
- Core/worker logs for each test execution
- Generated image artifacts and request payload metadata
- Updated docs in `gemini/docs/`
