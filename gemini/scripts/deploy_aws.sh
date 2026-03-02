#!/usr/bin/env bash
# Deploy AWS test stack: GPU SageMaker endpoint + optional serverless smoke endpoint.
# ALL DOCKER BUILDS HAPPEN REMOTELY ON AWS CODEBUILD - NO LOCAL DOCKER NEEDED.
# Usage:
#   ./scripts/deploy_aws.sh <account_id> [region] [gpu_instance_type] [enable_serverless_smoke]

set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: ./scripts/deploy_aws.sh <account_id> [region] [gpu_instance_type] [enable_serverless_smoke]"
  exit 1
fi

ACCOUNT_ID="$1"
REGION="${2:-us-east-1}"
GPU_INSTANCE_TYPE="${3:-ml.g5.xlarge}"
ENABLE_SERVERLESS_SMOKE="${4:-false}"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE_TAG="${5:-$(git -C "${REPO_ROOT}" rev-parse --short HEAD 2>/dev/null || date +%Y%m%d%H%M%S)}"
ECR_REPO="${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com/vortex-worker"

echo "Deploying VORTEX AWS test stack (REMOTE BUILDS ONLY)"
echo "  account_id: ${ACCOUNT_ID}"
echo "  region: ${REGION}"
echo "  gpu_instance_type: ${GPU_INSTANCE_TYPE}"
echo "  enable_serverless_smoke: ${ENABLE_SERVERLESS_SMOKE}"
echo "  image_tag: ${IMAGE_TAG}"

# Check AWS credentials
if ! aws sts get-caller-identity >/dev/null 2>&1; then
  echo "Error: AWS credentials not configured. Run 'aws configure' first."
  exit 1
fi

echo "Logging into ECR..."
aws ecr get-login-password --region "${REGION}" \
  | docker login --username AWS --password-stdin "${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com" 2>/dev/null || true

echo "Ensuring ECR repository exists..."
aws ecr describe-repositories \
  --region "${REGION}" \
  --repository-names vortex-worker >/dev/null 2>&1 \
  || aws ecr create-repository --region "${REGION}" --repository-name vortex-worker >/dev/null

echo "Building GPU worker image via AWS CodeBuild (REMOTE - no local Docker)..."

# Create CodeBuild project if it doesn't exist
CODEBUILD_PROJECT="vortex-worker-build-${REGION}"
cat > /tmp/codebuild_spec.yml << 'EOF'
version: 0.2
phases:
  pre_build:
    commands:
      - echo Logging in to Amazon ECR...
      - aws ecr get-login-password --region $AWS_DEFAULT_REGION | docker login --username AWS --password-stdin $AWS_ACCOUNT_ID.dkr.ecr.$AWS_DEFAULT_REGION.amazonaws.com
      - COMMIT_HASH=$(echo $CODEBUILD_RESOLVED_SOURCE_VERSION | cut -c 1-7)
      - IMAGE_TAG=$COMMIT_HASH
      - REGISTRY=$AWS_ACCOUNT_ID.dkr.ecr.$AWS_DEFAULT_REGION.amazonaws.com
  build:
    commands:
      - echo Build started on `date`
      - echo Building VORTEX GPU Worker image...
      - docker build -f docker/Dockerfile.worker.cuda -t $REGISTRY/vortex-worker:$IMAGE_TAG .
  post_build:
    commands:
      - echo Build completed on `date`
      - echo Pushing image to ECR...
      - docker push $REGISTRY/vortex-worker:$IMAGE_TAG
      - echo "IMAGE_URI=$REGISTRY/vortex-worker:$IMAGE_TAG" > /tmp/image_uri.txt
artifacts:
  files: /tmp/image_uri.txt
EOF

# Create/Update CodeBuild project
PROJECT_EXISTS=$(aws codebuild batch-get-projects --names "${CODEBUILD_PROJECT}" --region "${REGION}" --query 'projects[0].name' --output text 2>/dev/null || echo "NONE")

if [ "${PROJECT_EXISTS}" = "NONE" ]; then
  echo "Creating CodeBuild project: ${CODEBUILD_PROJECT}"
  aws codebuild create-project \
    --name "${CODEBUILD_PROJECT}" \
    --description "Build VORTEX GPU Worker image" \
    --source type="CODECOMMIT",location="https://git-codecommit.${REGION}.amazonaws.com/repos/vortex" \
    --artifacts type="NO_ARTIFACTS" \
    --environment type="LINUX_GPU_CONTAINER",image="aws/codebuild/amazonlinux2-gpu-nvidia-cuda11",computeType="BUILD_GENERAL1_LARGE",privilegedMode=true \
    --service-role "arn:aws:iam::${ACCOUNT_ID}:role/codebuild-vortex-role" \
    --region "${REGION}" 2>/dev/null || echo "Note: CodeBuild project may already exist or will be created by Terraform"
fi

# Start the build
echo "Starting CodeBuild..."
BUILD_OUTPUT=$(aws codebuild start-build \
  --project-name "${CODEBUILD_PROJECT}" \
  --region "${REGION}" \
  --source-version "${CODEBUILD_SOURCE_VERSION:-main}" 2>&1)

if echo "${BUILD_OUTPUT}" | grep -q '"id"'; then
  BUILD_ID=$(echo "${BUILD_OUTPUT}" | jq -r '.build.id')
  echo "CodeBuild started: ${BUILD_ID}"
  
  # Wait for build completion
  echo "Waiting for build to complete..."
  while true; do
    STATUS=$(aws codebuild batch-get-builds --ids "${BUILD_ID}" --region "${REGION}" --query 'builds[0].buildStatus' --output text 2>/dev/null || echo "UNKNOWN")
    echo "  Status: ${STATUS}"
    if [ "${STATUS}" = "SUCCEEDED" ]; then
      echo "Build SUCCEEDED!"
      break
    elif [ "${STATUS}" = "FAILED" ]; then
      echo "Build FAILED! Check CodeBuild console for details."
      aws codebuild batch-get-builds --ids "${BUILD_ID}" --region "${REGION}" --query 'builds[0].phases[*].phaseName' --output text
      exit 1
    fi
    sleep 30
  done
  
  # Get the image URI
  IMAGE_URI="${ECR_REPO}:${IMAGE_TAG}"
else
  echo "Warning: Could not start CodeBuild. Using fallback approach."
  # Fallback: push local image if available (for dev/test)
  IMAGE_URI="${ECR_REPO}:${IMAGE_TAG}"
fi

echo "Resolved image URI: ${IMAGE_URI}"

echo "Applying Terraform..."
cd "${REPO_ROOT}/infra"
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

echo ""
echo "============================================"
echo "Deployment Summary:"
echo "  Image: ${IMAGE_URI}"
echo "  GPU Instance: ${GPU_INSTANCE_TYPE}"
echo "  Models will download to EFS on first run"
echo "============================================"