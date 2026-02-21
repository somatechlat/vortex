output "vpc_id" {
  description = "The ID of the VORTEX VPC"
  value       = module.vpc.vpc_id
}

output "worker_sg_id" {
  description = "Security Group ID for GPU Workers"
  value       = aws_security_group.worker.id
}

output "efs_dns_name" {
  description = "DNS name of the EFS filesystem for model cache"
  value       = aws_efs_file_system.models.dns_name
}

output "private_subnets" {
  description = "List of private subnet IDs"
  value       = module.vpc.private_subnets
}

output "nat_public_ips" {
  description = "List of public Elastic IPs created for AWS NAT Gateway"
  value       = module.vpc.nat_public_ips
}

# output "asg_name" - REMOVED (Legacy EC2)


output "sagemaker_endpoint_name" {
  description = "Name of the SageMaker GPU real-time endpoint"
  value       = aws_sagemaker_endpoint.vortex.name
}

output "sagemaker_serverless_smoke_endpoint_name" {
  description = "Name of optional SageMaker serverless smoke endpoint"
  value       = var.enable_sagemaker_serverless_smoke ? aws_sagemaker_endpoint.vortex_serverless_smoke[0].name : null
}

output "ecr_worker_repository_url" {
  description = "ECR repository URL for vortex-worker images"
  value       = aws_ecr_repository.vortex_worker.repository_url
}
