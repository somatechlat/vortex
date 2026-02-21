# VORTEX Deployment and Real Testing Runbook
Date: 2026-02-21  
Scope: CPU-local execution + AWS GPU execution

## 1. Target Server Split

- CPU layer:
  - Local machine: Docker Compose stack for control-plane development and smoke tests.
  - AWS: ECS Fargate for API/control workers (serverless containers where possible).
- GPU layer:
  - AWS SageMaker real-time endpoint for real diffusion execution.
- Optional smoke-only layer:
  - SageMaker serverless endpoint for API contract checks only.

## 2. What Is Already Verified

- `cargo check --workspace` passes.
- `bun run check` passes with zero errors/warnings.
- `terraform validate` in `gemini/infra` passes.
- `docker compose config` in `gemini/infra/docker/standalone` passes.
- `vortex-core` binary compiles and starts; direct host run requires DB env wiring.

## 3. Local CPU Bring-Up (Real Commands)

```bash
cd gemini/infra/docker/standalone
docker compose up -d --build
docker compose ps
```

Health checks:

```bash
curl -fsS http://localhost:11188/health
curl -fsS http://localhost:11188/info
```

UI:

```bash
open http://localhost:11100
```

## 4. AWS GPU Deployment (Real Commands)

Prereqs:
- authenticated AWS CLI profile
- Docker daemon running
- Terraform installed

Deploy:

```bash
./gemini/scripts/deploy_aws.sh <ACCOUNT_ID> us-east-1 ml.g5.xlarge false
```

This script:
1. Builds/pushes GPU worker image to ECR.
2. Resolves image digest.
3. Applies Terraform with immutable `sagemaker_worker_image_uri` (`@sha256:...`).

Read outputs:

```bash
cd gemini/infra
terraform output
```

## 5. Real Endpoint Invocation Examples

### 5.1 Ping test (endpoint container health)

```bash
aws sagemaker-runtime invoke-endpoint \
  --region us-east-1 \
  --endpoint-name vortex-gpu-test-endpoint \
  --content-type application/json \
  --body '{"op_type":"noop","params":{},"inputs":{}}' \
  /tmp/vortex_out.json
cat /tmp/vortex_out.json
```

### 5.2 Load model (stateful warm-up on endpoint container)

`/tmp/load_model.json`
```json
{
  "op_type": "Loader::Checkpoint",
  "params": { "ckpt_name": "sd15" },
  "inputs": {}
}
```

Invoke:
```bash
aws sagemaker-runtime invoke-endpoint \
  --region us-east-1 \
  --endpoint-name vortex-gpu-test-endpoint \
  --content-type application/json \
  --body fileb:///tmp/load_model.json \
  /tmp/load_model_out.json
cat /tmp/load_model_out.json
```

### 5.3 Generate latent sample (real prompt)

`/tmp/sample_latent.json`
```json
{
  "op_type": "Sampler::KSampler",
  "params": {
    "prompt": "A condor flying low and landing on water, cinematic advertising shot, high detail",
    "negative_prompt": "blurry, low quality, artifacts",
    "steps": 20,
    "cfg": 7.0,
    "seed": 42
  },
  "inputs": {}
}
```

Invoke:
```bash
aws sagemaker-runtime invoke-endpoint \
  --region us-east-1 \
  --endpoint-name vortex-gpu-test-endpoint \
  --content-type application/json \
  --body fileb:///tmp/sample_latent.json \
  /tmp/sample_latent_out.json
```

### 5.4 Decode latent to image tensor

Use the `outputs.samples` object from `/tmp/sample_latent_out.json` and place it into `inputs.samples`:

`/tmp/decode_image.json`
```json
{
  "op_type": "Decoder::VAE",
  "params": {},
  "inputs": {
    "samples": {
      "data": "<BASE64_FROM_PREVIOUS_STEP>",
      "shape": [1, 4, 64, 64],
      "dtype": "float16"
    }
  }
}
```

Invoke:
```bash
aws sagemaker-runtime invoke-endpoint \
  --region us-east-1 \
  --endpoint-name vortex-gpu-test-endpoint \
  --content-type application/json \
  --body fileb:///tmp/decode_image.json \
  /tmp/decode_image_out.json
```

## 6. Determinism and Latency Test Set

Run each prompt 5 times with fixed seed on same endpoint:
- Prompt A: condor landing on water (commercial style)
- Prompt B: product studio shot (waterproof shoes)
- Prompt C: action scene with reflective water

Collect:
- total latency
- model warm/cold latency split
- output checksum for deterministic mode checks

## 7. AWS Observability Checks

CloudWatch logs:
```bash
aws logs describe-log-groups --region us-east-1 | rg vortex
```

SageMaker endpoint status:
```bash
aws sagemaker describe-endpoint \
  --region us-east-1 \
  --endpoint-name vortex-gpu-test-endpoint \
  --query 'EndpointStatus'
```

## 8. Go/No-Go Gates

- Gate 1: Local CPU stack healthy and API responsive.
- Gate 2: GPU endpoint `InService`.
- Gate 3: Real prompt returns valid output payload.
- Gate 4: Repeat runs complete without endpoint errors.
- Gate 5: Latency/quality evidence captured and archived.
