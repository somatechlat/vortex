provider "aws" {
  region = var.aws_region
}

# -----------------------------------------------------------------------------
# VPC & Network
# -----------------------------------------------------------------------------

module "vpc" {
  source = "terraform-aws-modules/vpc/aws"

  name = "vortex-vpc"
  cidr = "10.0.0.0/16"

  azs             = ["${var.aws_region}a", "${var.aws_region}b"]
  private_subnets = ["10.0.1.0/24", "10.0.2.0/24"]
  public_subnets  = ["10.0.101.0/24", "10.0.102.0/24"]

  enable_nat_gateway = true
  single_nat_gateway = true
}

# -----------------------------------------------------------------------------
# Security Groups
# -----------------------------------------------------------------------------

resource "aws_security_group" "worker" {
  name        = "vortex-worker-sg"
  description = "Security group for VORTEX GPU Workers"
  vpc_id      = module.vpc.vpc_id

  # Allow inbound control from VPN/NLB
  ingress {
    from_port   = 11190
    to_port     = 11190
    protocol    = "tcp"
    cidr_blocks = ["10.0.0.0/16"]
  }

  # Allow outbound to everywhere (pip/docker pull)
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

resource "aws_security_group" "efs" {
  name        = "vortex-efs-sg"
  description = "Security group for VORTEX Model Cache"
  vpc_id      = module.vpc.vpc_id

  # Allow NFS from Worker SG
  ingress {
    from_port       = 2049
    to_port         = 2049
    protocol        = "tcp"
    security_groups = [aws_security_group.worker.id]
  }
}

# -----------------------------------------------------------------------------
# Storage (EFS for Model Cache)
# -----------------------------------------------------------------------------

resource "aws_efs_file_system" "models" {
  creation_token  = "vortex-models"
  encrypted       = true
  throughput_mode = "bursting"
}

resource "aws_efs_mount_target" "models" {
  count           = length(module.vpc.private_subnets)
  file_system_id  = aws_efs_file_system.models.id
  subnet_id       = module.vpc.private_subnets[count.index]
  security_groups = [aws_security_group.efs.id]
}

# -----------------------------------------------------------------------------
# Compute (Auto Scaling Group)
# -----------------------------------------------------------------------------

# -----------------------------------------------------------------------------
# Compute (Auto Scaling Group) - SUPERSEDED BY SERVERLESS STACK
# -----------------------------------------------------------------------------

# Legacy EC2 references removed to support Serverless Pivot.
# See main_serverless.tf for Fargate/SageMaker configuration.

# -----------------------------------------------------------------------------
# Storage (S3 for Model Repository)
# -----------------------------------------------------------------------------

resource "aws_s3_bucket" "vortex_models" {
  bucket        = "vortex-models-${var.aws_region}-${var.account_id}"
  force_destroy = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "vortex_models_enc" {
  bucket = aws_s3_bucket.vortex_models.id

  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "vortex_models_block" {
  bucket = aws_s3_bucket.vortex_models.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}
