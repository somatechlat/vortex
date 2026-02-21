# -----------------------------------------------------------------------------
# Fargate & ECR (Serverless Compute)
# -----------------------------------------------------------------------------

resource "aws_ecr_repository" "vortex_worker" {
  name                 = "vortex-worker"
  image_tag_mutability = "IMMUTABLE"

  image_scanning_configuration {
    scan_on_push = true
  }
}

locals {
  worker_image_uri = var.sagemaker_worker_image_uri
}

resource "aws_ecs_cluster" "vortex" {
  name = "vortex-serverless-cluster"
}

resource "aws_ecs_cluster_capacity_providers" "vortex" {
  cluster_name = aws_ecs_cluster.vortex.name

  capacity_providers = ["FARGATE", "FARGATE_SPOT"]

  default_capacity_provider_strategy {
    base              = 1
    weight            = 100
    capacity_provider = "FARGATE_SPOT"
  }
}

# -----------------------------------------------------------------------------
# IAM & Roles
# -----------------------------------------------------------------------------

resource "aws_iam_role" "ecs_execution" {
  name = "vortex-ecs-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "ecs-tasks.amazonaws.com"
      }
    }]
  })
}

resource "aws_iam_role_policy_attachment" "ecs_execution_serverless" {
  role       = aws_iam_role.ecs_execution.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

# -----------------------------------------------------------------------------
# SageMaker (GPU Real-Time + Optional Serverless Smoke)
# -----------------------------------------------------------------------------

resource "aws_iam_role" "sagemaker_execution" {
  name = "vortex-sagemaker-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "sagemaker.amazonaws.com"
      }
    }]
  })
}

resource "aws_iam_role_policy" "sagemaker_execution_policy" {
  role = aws_iam_role.sagemaker_execution.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "ecr:GetAuthorizationToken"
        ]
        Resource = "*"
      },
      {
        Effect = "Allow"
        Action = [
          "ecr:BatchCheckLayerAvailability",
          "ecr:GetDownloadUrlForLayer",
          "ecr:BatchGetImage"
        ]
        Resource = aws_ecr_repository.vortex_worker.arn
      },
      {
        Effect = "Allow"
        Action = [
          "logs:CreateLogStream",
          "logs:PutLogEvents",
          "logs:CreateLogGroup",
          "logs:DescribeLogStreams",
          "logs:DescribeLogGroups"
        ]
        Resource = "*"
      },
      {
        Effect = "Allow"
        Action = [
          "s3:GetObject",
          "s3:PutObject",
          "s3:ListBucket"
        ]
        Resource = [
          aws_s3_bucket.vortex_models.arn,
          "${aws_s3_bucket.vortex_models.arn}/*"
        ]
      }
    ]
  })
}

resource "aws_sagemaker_model" "vortex" {
  name               = "vortex-gpu-test-model"
  execution_role_arn = aws_iam_role.sagemaker_execution.arn
  depends_on         = [aws_iam_role_policy.sagemaker_execution_policy]

  lifecycle {
    precondition {
      condition     = length(trimspace(local.worker_image_uri)) > 0 && can(regex("@sha256:[a-f0-9]{64}$", local.worker_image_uri))
      error_message = "sagemaker_worker_image_uri must be set to an immutable ECR digest URI (example: <repo>@sha256:<digest>)."
    }
  }

  primary_container {
    image = local.worker_image_uri
    mode  = "SingleModel"
  }
}

resource "aws_sagemaker_endpoint_configuration" "realtime_gpu" {
  name = "vortex-gpu-test-endpoint-config"

  production_variants {
    variant_name           = "AllTraffic"
    model_name             = aws_sagemaker_model.vortex.name
    instance_type          = var.sagemaker_gpu_instance_type
    initial_instance_count = var.sagemaker_initial_instance_count
  }
}

resource "aws_sagemaker_endpoint" "vortex" {
  name                 = "vortex-gpu-test-endpoint"
  endpoint_config_name = aws_sagemaker_endpoint_configuration.realtime_gpu.name
}

# Optional serverless smoke endpoint for API-contract testing (CPU, not for real diffusion loads)
resource "aws_sagemaker_endpoint_configuration" "serverless_smoke" {
  count = var.enable_sagemaker_serverless_smoke ? 1 : 0
  name  = "vortex-serverless-smoke-endpoint-config"

  production_variants {
    variant_name = "AllTraffic"
    model_name   = aws_sagemaker_model.vortex.name
    serverless_config {
      max_concurrency   = var.sagemaker_serverless_max_concurrency
      memory_size_in_mb = var.sagemaker_serverless_memory_mb
    }
  }
}

resource "aws_sagemaker_endpoint" "vortex_serverless_smoke" {
  count                = var.enable_sagemaker_serverless_smoke ? 1 : 0
  name                 = "vortex-serverless-smoke-endpoint"
  endpoint_config_name = aws_sagemaker_endpoint_configuration.serverless_smoke[0].name
}

# -----------------------------------------------------------------------------
# AWS CodeBuild (Serverless Build Pipeline)
# -----------------------------------------------------------------------------

resource "aws_codebuild_project" "vortex_build" {
  name          = "vortex-worker-build"
  description   = "Builds the VORTEX Worker GPU Docker image"
  build_timeout = "60" # 60 minutes
  service_role  = aws_iam_role.codebuild_role.arn

  artifacts {
    type = "NO_ARTIFACTS"
  }

  environment {
    compute_type                = "BUILD_GENERAL1_LARGE" # 15GB RAM, 8 vCPUs (for faster concurrent compiles)
    image                       = "aws/codebuild/standard:7.0"
    type                        = "LINUX_CONTAINER"
    image_pull_credentials_type = "CODEBUILD"
    privileged_mode             = true # Required for Docker build

    environment_variable {
      name  = "AWS_DEFAULT_REGION"
      value = var.aws_region
    }
    environment_variable {
      name  = "AWS_ACCOUNT_ID"
      value = var.account_id
    }
  }

  source {
    type            = "GITHUB"
    location        = "https://github.com/somatechlat/vortex.git"
    git_clone_depth = 1

    buildspec = "gemini/buildspec.yml"
  }

  logs_config {
    cloudwatch_logs {
      group_name  = "/aws/codebuild/vortex-worker"
      stream_name = "build-log"
    }
  }
}

# -----------------------------------------------------------------------------
# CodeBuild IAM Role
# -----------------------------------------------------------------------------

resource "aws_iam_role" "codebuild_role" {
  name = "vortex-codebuild-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [{
      Action = "sts:AssumeRole"
      Effect = "Allow"
      Principal = {
        Service = "codebuild.amazonaws.com"
      }
    }]
  })
}

resource "aws_iam_role_policy" "codebuild_policy" {
  role = aws_iam_role.codebuild_role.name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Resource = [
          "*"
        ]
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "ecr:GetAuthorizationToken"
        ]
        Resource = "*"
      },
      {
        Effect = "Allow"
        Action = [
          "ecr:BatchCheckLayerAvailability",
          "ecr:CompleteLayerUpload",
          "ecr:GetAuthorizationToken",
          "ecr:InitiateLayerUpload",
          "ecr:PutImage",
          "ecr:UploadLayerPart"
        ]
        Resource = aws_ecr_repository.vortex_worker.arn
      }
    ]
  })
}
