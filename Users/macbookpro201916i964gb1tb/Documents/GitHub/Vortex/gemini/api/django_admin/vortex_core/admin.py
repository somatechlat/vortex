"""
VORTEX Django Admin - Read-Only Monitoring Interface

Provides admin views for monitoring Rust execution engine.
All actions are read-only - write operations must go through Rust API.
"""

from django.contrib import admin
from django.utils.html import format_html
from .models import Run, RunStep, Graph, Tenant, ModelEntry


@admin.register(Run)
class RunAdmin(admin.ModelAdmin):
    """Admin for Rust execution runs."""
    
    list_display = ["id", "status_colored", "graph_hash", "created_at_formatted", "duration"]
    list_filter = ["status", "created_at"]
    search_fields = ["id", "graph_hash"]
    readonly_fields = [field.name for field in Run._meta.fields]
    actions = None  # No bulk actions - read-only
    
    def has_add_permission(self, request):
        return False
    
    def has_change_permission(self, request, obj=None):
        return False
    
    def has_delete_permission(self, request, obj=None):
        return False
    
    def status_colored(self, obj):
        colors = {
            "PENDING": "gray",
            "RUNNING": "blue",
            "COMPLETED": "green",
            "FAILED": "red",
        }
        return format_html(
            '<span style="color: {}; font-weight: bold;">{}</span>',
            colors.get(obj.status, "black"),
            obj.status
        )
    status_colored.short_description = "Status"
    
    def created_at_formatted(self, obj):
        import datetime
        return datetime.datetime.fromtimestamp(obj.created_at).strftime("%Y-%m-%d %H:%M:%S")
    created_at_formatted.short_description = "Created"
    
    def duration(self, obj):
        if obj.completed_at:
            import datetime
            diff = obj.completed_at - obj.created_at
            return str(datetime.timedelta(seconds=diff))
        return "Running..."
    duration.short_description = "Duration"


@admin.register(RunStep)
class RunStepAdmin(admin.ModelAdmin):
    """Admin for execution step metrics."""
    
    list_display = ["run_id", "node_id", "worker_pid", "duration_ms", "peak_vram_mb"]
    list_filter = ["worker_pid"]
    search_fields = ["run_id", "node_id"]
    readonly_fields = [field.name for field in RunStep._meta.fields]
    actions = None
    
    def has_add_permission(self, request):
        return False
    
    def has_change_permission(self, request, obj=None):
        return False
    
    def has_delete_permission(self, request, obj=None):
        return False
    
    def duration_ms(self, obj):
        return f"{obj.duration_us / 1000:.2f} ms"
    duration_ms.short_description = "Duration"


@admin.register(Graph)
class GraphAdmin(admin.ModelAdmin):
    """Admin for workflow graphs."""
    
    list_display = ["name", "version", "tenant_id", "created_at_formatted"]
    search_fields = ["name", "id"]
    readonly_fields = [field.name for field in Graph._meta.fields]
    actions = None
    
    def has_add_permission(self, request):
        return False
    
    def has_change_permission(self, request, obj=None):
        return False
    
    def has_delete_permission(self, request, obj=None):
        return False
    
    def created_at_formatted(self, obj):
        import datetime
        return datetime.datetime.fromtimestamp(obj.created_at).strftime("%Y-%m-%d %H:%M:%S")
    created_at_formatted.short_description = "Created"


@admin.register(Tenant)
class TenantAdmin(admin.ModelAdmin):
    """Admin for tenant management (read-only)."""
    
    list_display = ["name", "slug", "tier", "status", "max_concurrent_jobs"]
    list_filter = ["tier", "status"]
    search_fields = ["name", "slug"]
    readonly_fields = [field.name for field in Tenant._meta.fields]
    actions = None
    
    def has_add_permission(self, request):
        return False
    
    def has_change_permission(self, request, obj=None):
        return False
    
    def has_delete_permission(self, request, obj=None):
        return False


@admin.register(ModelEntry)
class ModelEntryAdmin(admin.ModelAdmin):
    """Admin for model registry."""
    
    list_display = ["name", "model_type", "source_repo", "size_mb", "last_used_formatted"]
    list_filter = ["model_type", "source_type"]
    search_fields = ["name", "source_repo"]
    readonly_fields = [field.name for field in ModelEntry._meta.fields]
    actions = None
    
    def has_add_permission(self, request):
        return False
    
    def has_change_permission(self, request, obj=None):
        return False
    
    def has_delete_permission(self, request, obj=None):
        return False
    
    def size_mb(self, obj):
        if obj.size_bytes:
            return f"{obj.size_bytes / (1024*1024):.2f} MB"
        return "-"
    size_mb.short_description = "Size"
    
    def last_used_formatted(self, obj):
        if obj.last_used:
            import datetime
            return datetime.datetime.fromtimestamp(obj.last_used).strftime("%Y-%m-%d %H:%M:%S")
        return "Never"
    last_used_formatted.short_description = "Last Used"
