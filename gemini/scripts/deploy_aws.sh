#!/usr/bin/env bash
# Deploy AWS test stack: GPU SageMaker endpoint + optional serverless smoke endpoint.
# Usage:
#   ./gemini/scripts/deploy_aws.sh <account_id> [region] [gpu_instance_type] [enable_serverless_smoke]

set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: ./gemini/scripts/deploy_aws.sh <account_id> [region] [gpu_instance_type] [enable_serverless_smoke] [image_tag]"
  exit 1
fi

ACCOUNT_ID="$1"
REGION="${2:-us-east-1}"
GPU_INSTANCE_TYPE="${3:-ml.g5.xlarge}"
ENABLE_SERVERLESS_SMOKE="${4:-false}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
IMAGE_TAG="${5:-$(git -C "${REPO_ROOT}" rev-parse --short HEAD 2>/dev/null || date +%Y%m%d%H%M%S)}"
ECR_REPO="${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com/vortex-worker"

for cmd in aws docker terraform; do
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "Missing dependency: ${cmd}"
    exit 1
  fi
done

echo "Deploying VORTEX AWS test stack"
echo "  account_id: ${ACCOUNT_ID}"
echo "  region: ${REGION}"
echo "  gpu_instance_type: ${GPU_INSTANCE_TYPE}"
echo "  enable_serverless_smoke: ${ENABLE_SERVERLESS_SMOKE}"
echo "  image_tag: ${IMAGE_TAG}"

echo "Logging into ECR..."
aws ecr get-login-password --region "${REGION}" \
  | docker login --username AWS --password-stdin "${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com"

echo "Ensuring ECR repository exists..."
aws ecr describe-repositories \
  --region "${REGION}" \
  --repository-names vortex-worker >/dev/null 2>&1 \
  || aws ecr create-repository --region "${REGION}" --repository-name vortex-worker >/dev/null

echo "Building GPU worker image..."
docker buildx build \
  --platform linux/amd64 \
  --provenance=false \
  -t "${ECR_REPO}:${IMAGE_TAG}" \
  -f "${REPO_ROOT}/gemini/docker/Dockerfile.worker.cuda" \
  --push \
  "${REPO_ROOT}"

IMAGE_DIGEST="$(aws ecr describe-images --region "${REGION}" --repository-name vortex-worker --image-ids imageTag="${IMAGE_TAG}" --query 'imageDetails[0].imageDigest' --output text)"
IMAGE_URI="${ECR_REPO}@${IMAGE_DIGEST}"
echo "Resolved immutable image URI: ${IMAGE_URI}"

echo "Applying Terraform..."
cd "${REPO_ROOT}/gemini/infra"
terraform init
terraform apply \
  -var="account_id=${ACCOUNT_ID}" \
  -var="aws_region=${REGION}" \
  -var="sagemaker_gpu_instance_type=${GPU_INSTANCE_TYPE}" \
  -var="enable_sagemaker_serverless_smoke=${ENABLE_SERVERLESS_SMOKE}" \
  -var="sagemaker_worker_image_uri=${IMAGE_URI}" \
  -auto-approve

echo "Deployment complete. Terraform outputs:"
terraform output
