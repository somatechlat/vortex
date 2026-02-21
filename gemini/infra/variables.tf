variable "aws_region" {
  description = "AWS region to deploy resources (e.g., us-east-1)"
  type        = string
  default     = "us-east-1"
}

variable "account_id" {
  description = "AWS Account ID for ECR access"
  type        = string
}

variable "vpc_cidr" {
  description = "VPC CIDR block"
  type        = string
  default     = "10.0.0.0/16"
}

variable "ami_id" {
  description = "AMI ID for Deep Learning AMI GPU PyTorch 2.0 (Ubuntu 22.04)"
  type        = string
  # This default is for us-east-1 (check latest)
  default = "ami-053b0d53c279acc90"
}

variable "instance_type" {
  description = "Instance type for GPU Workers"
  type        = string
  default     = "g5.xlarge"
}

variable "min_workers" {
  description = "Minimum number of GPU workers"
  type        = number
  default     = 0
}

variable "max_workers" {
  description = "Maximum number of GPU workers"
  type        = number
  default     = 5
}

variable "sagemaker_gpu_instance_type" {
  description = "SageMaker real-time GPU instance type for diffusion testing"
  type        = string
  default     = "ml.g5.xlarge"
}

variable "sagemaker_initial_instance_count" {
  description = "Initial number of instances for the GPU real-time endpoint"
  type        = number
  default     = 1
}

variable "enable_sagemaker_serverless_smoke" {
  description = "Create an optional serverless endpoint for API-contract smoke tests"
  type        = bool
  default     = false
}

variable "sagemaker_serverless_max_concurrency" {
  description = "Max concurrency for optional SageMaker serverless smoke endpoint"
  type        = number
  default     = 2
}

variable "sagemaker_serverless_memory_mb" {
  description = "Memory size for optional SageMaker serverless smoke endpoint"
  type        = number
  default     = 4096
}

variable "sagemaker_worker_image_uri" {
  description = "Immutable worker image URI in ECR digest form (required): <account>.dkr.ecr.<region>.amazonaws.com/vortex-worker@sha256:<digest>"
  type        = string
  default     = ""
}
