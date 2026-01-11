"""
VORTEX Django Models - Read-Only Mirror of Rust SeaORM Entities

These models mirror the Rust entities for Django admin read-only access.
CRITICAL: Set managed=False to prevent Django migrations from conflicting with Rust.
"""

from django.db import models


class Run(models.Model):
    """Mirror of Rust: crates/vortex-core/src/entities.rs - run::Model"""
    
    id = models.CharField(max_length=36, primary_key=True)
    graph_hash = models.CharField(max_length=255)
    status = models.CharField(max_length=16)  # Pending, Running, Completed, Failed
    created_at = models.BigIntegerField()
    completed_at = models.BigIntegerField(null=True, blank=True)
    error_json = models.TextField(null=True, blank=True)
    
    class Meta:
        managed = False  # CRITICAL: Rust owns this table
        db_table = "runs"
        verbose_name = "Run"
        verbose_name_plural = "Runs"
    
    def __str__(self):
        return f"{self.id} - {self.status}"


class RunStep(models.Model):
    """Mirror of Rust: crates/vortex-core/src/entities.rs - run_step::Model"""
    
    run_id = models.CharField(max_length=36)
    node_id = models.CharField(max_length=255)
    worker_pid = models.IntegerField()
    duration_us = models.BigIntegerField()
    peak_vram_mb = models.BigIntegerField()
    
    class Meta:
        managed = False  # CRITICAL: Rust owns this table
        db_table = "run_steps"
        verbose_name = "Run Step"
        verbose_name_plural = "Run Steps"
        # Composite primary key in Django
        unique_together = ["run_id", "node_id"]
    
    def __str__(self):
        return f"{self.run_id}:{self.node_id}"


class Graph(models.Model):
    """Mirror of Rust: crates/vortex-core/src/entities.rs - graph::Model"""
    
    id = models.CharField(max_length=36, primary_key=True)
    tenant_id = models.CharField(max_length=36)
    name = models.CharField(max_length=255)
    version = models.IntegerField()
    graph_json = models.TextField()
    created_at = models.BigIntegerField()
    updated_at = models.BigIntegerField()
    
    class Meta:
        managed = False
        db_table = "graphs"
        verbose_name = "Graph"
        verbose_name_plural = "Graphs"
    
    def __str__(self):
        return f"{self.name} (v{self.version})"


class Tenant(models.Model):
    """Mirror of Rust: crates/vortex-core/src/entities.rs - tenant::Model"""
    
    TIER_CHOICES = [
        ("free", "Free"),
        ("pro", "Pro"),
        ("enterprise", "Enterprise"),
    ]
    
    STATUS_CHOICES = [
        ("provisioning", "Provisioning"),
        ("active", "Active"),
        ("suspended", "Suspended"),
        ("pending_deletion", "Pending Deletion"),
        ("deleted", "Deleted"),
    ]
    
    id = models.CharField(max_length=36, primary_key=True)
    name = models.CharField(max_length=255)
    slug = models.SlugField(unique=True)
    tier = models.CharField(max_length=16, choices=TIER_CHOICES)
    status = models.CharField(max_length=16, choices=STATUS_CHOICES)
    max_concurrent_jobs = models.IntegerField()
    max_gpu_hours_month = models.BigIntegerField()
    max_graphs = models.IntegerField()
    max_models = models.IntegerField()
    max_members = models.IntegerField()
    max_storage_bytes = models.BigIntegerField()
    created_at = models.BigIntegerField()
    updated_at = models.BigIntegerField()
    
    class Meta:
        managed = False
        db_table = "tenants"
        verbose_name = "Tenant"
        verbose_name_plural = "Tenants"
    
    def __str__(self):
        return f"{self.name} ({self.tier})"


class ModelEntry(models.Model):
    """Mirror of Rust: crates/vortex-core/src/entities.rs - model_entry::Model"""
    
    MODEL_TYPE_CHOICES = [
        ("diffusers", "Diffusers"),
        ("checkpoint", "Checkpoint"),
        ("lora", "LoRA"),
        ("vae", "VAE"),
        ("controlnet", "ControlNet"),
    ]
    
    id = models.CharField(max_length=36, primary_key=True)
    name = models.CharField(max_length=255)
    source_type = models.CharField(max_length=255)
    source_repo = models.CharField(max_length=255)
    model_type = models.CharField(max_length=16, choices=MODEL_TYPE_CHOICES)
    size_bytes = models.BigIntegerField(null=True, blank=True)
    hash = models.CharField(max_length=255, null=True, blank=True)
    cached_path = models.CharField(max_length=4096, null=True, blank=True)
    last_used = models.BigIntegerField(null=True, blank=True)
    created_at = models.BigIntegerField()
    
    class Meta:
        managed = False
        db_table = "models"
        verbose_name = "Model"
        verbose_name_plural = "Models"
    
    def __str__(self):
        return self.name
