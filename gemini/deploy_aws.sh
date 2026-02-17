#!/bin/bash
# VORTEX AWS Serverless Deployment Script
# Usage: ./deploy_aws.sh <AWS_ACCOUNT_ID> <AWS_REGION>

set -e

if [ -z "$1" ]; then
    echo "Usage: ./deploy_aws.sh <AWS_ACCOUNT_ID> <AWS_REGION>"
    exit 1
fi

ACCOUNT_ID=$1
REGION=${2:-us-east-1}
ECR_REPO="${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com/vortex-worker"

echo "============================================================"
echo "🚀 VORTEX AWS DEPLOYMENT - SERVERLESS STACK"
echo "Region: $REGION | Account: $ACCOUNT_ID"
echo "============================================================"

# Check for AWS CLI
if ! command -v aws &> /dev/null; then
    echo "❌ AWS CLI not found. Please install it."
    exit 1
fi

# Check for Terraform
if ! command -v terraform &> /dev/null; then
    echo "❌ Terraform not found. Please install it."
    exit 1
fi

# 1. Login to ECR
echo -e "\n🔑 Logging into ECR..."
aws ecr get-login-password --region $REGION | docker login --username AWS --password-stdin ${ACCOUNT_ID}.dkr.ecr.${REGION}.amazonaws.com

# 2. Build Docker Image
echo -e "\n🐳 Building GPU Worker Image..."
docker build -t vortex-worker -f gemini/docker/Dockerfile.worker.cuda .

# 3. Tag and Push
echo -e "\npushing to ECR..."
docker tag vortex-worker:latest $ECR_REPO:latest
docker push $ECR_REPO:latest

# 4. Terraform Apply
echo -e "\n🌍 Provisioning Infrastructure (Terraform)..."
cd gemini/infra
terraform init
terraform apply -var="account_id=${ACCOUNT_ID}" -var="aws_region=${REGION}" -auto-approve

# 5. Output Summary
echo -e "\n✅ DEPLOYMENT COMPLETE"
terraform output

echo -e "\nTo update the SageMaker endpoint with new code, simply re-run this script."
