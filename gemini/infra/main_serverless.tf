# -----------------------------------------------------------------------------
# Fargate & ECR (Serverless Compute)
# -----------------------------------------------------------------------------

resource "aws_ecr_repository" "vortex_worker" {
  name                 = "vortex-worker"
  image_tag_mutability = "MUTABLE"

  image_scanning_configuration {
    scan_on_push = true
  }
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
# SageMaker (Serverless Inference)
# -----------------------------------------------------------------------------

resource "aws_sagemaker_model" "vortex" {
  name               = "vortex-serverless-model"
  execution_role_arn = aws_iam_role.ecs_execution.arn

  primary_container {
    image = "${aws_ecr_repository.vortex_worker.repository_url}:latest"
    mode  = "SingleModel"
  }
}

resource "aws_sagemaker_endpoint_configuration" "serverless" {
  name = "vortex-serverless-endpoint-config"

  production_variants {
    variant_name           = "AllTraffic"
    model_name             = aws_sagemaker_model.vortex.name

    serverless_config {
      max_concurrency = 5
      memory_size_in_mb = 6144
    }
  }
}

resource "aws_sagemaker_endpoint" "vortex" {
  name                 = "vortex-serverless-endpoint"
  endpoint_config_name = aws_sagemaker_endpoint_configuration.serverless.name
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
      value = "us-east-1"
    }
    environment_variable {
      name  = "AWS_ACCOUNT_ID"
      value = "575271901235"
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
