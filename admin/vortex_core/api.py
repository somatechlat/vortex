from ninja import NinjaAPI, Schema
from ninja.security import HttpBearer
from ninja.pagination import paginate, LimitOffsetPagination
from typing import List, Optional, Dict, Any
from django.conf import settings
from .models import Run, RunStep, Graph, Tenant, ModelEntry
from .mcp import mcp_service
from admin.common.messages import get_message
import logging

logger = logging.getLogger(__name__)


# ═══════════════════════════════════════════════════════════════
#                    AUTHENTICATION
# ═══════════════════════════════════════════════════════════════


class VortexBearerAuth(HttpBearer):
    """Bearer token auth for Django Ninja API.

    Validates against the VORTEX_ADMIN_API_TOKEN environment variable.
    In production, this should be replaced with Keycloak OIDC validation.
    """

    def authenticate(self, request, token: str):
        expected = getattr(settings, "VORTEX_ADMIN_API_TOKEN", None)
        if not expected:
            import os
            expected = os.environ.get("VORTEX_ADMIN_API_TOKEN")
        if not expected:
            logger.error(get_message("ERROR_UNAUTHORIZED", resource="admin API (no token configured)"))
            return None
        if token == expected:
            return token
        logger.warning(get_message("ERROR_UNAUTHORIZED", resource="admin API"))
        return None


auth = VortexBearerAuth()

api = NinjaAPI(title="VORTEX Core API", version="3.0.0", auth=auth)


# ═══════════════════════════════════════════════════════════════
#                    SCHEMAS
# ═══════════════════════════════════════════════════════════════


class RunSchema(Schema):
    id: str
    graph_hash: str
    status: str
    created_at: int
    completed_at: Optional[int] = None
    error_json: Optional[str] = None

class RunStepSchema(Schema):
    run_id: str
    node_id: str
    worker_pid: int
    duration_us: int
    peak_vram_mb: int

class GraphSchema(Schema):
    id: str
    tenant_id: str
    name: str
    version: int
    graph_json: str
    created_at: int
    updated_at: int

class TenantSchema(Schema):
    id: str
    name: str
    slug: str
    tier: str
    status: str
    max_concurrent_jobs: int
    max_gpu_hours_month: int
    max_graphs: int
    max_models: int
    max_members: int
    max_storage_bytes: int
    created_at: int
    updated_at: int

class ModelEntrySchema(Schema):
    id: str
    name: str
    source_type: str
    source_repo: str
    model_type: str
    size_bytes: Optional[int] = None
    hash: Optional[str] = None
    cached_path: Optional[str] = None
    last_used: Optional[int] = None
    created_at: int


# ═══════════════════════════════════════════════════════════════
#                    ENDPOINTS (all authenticated + paginated)
# ═══════════════════════════════════════════════════════════════


@api.get("/runs", response=List[RunSchema])
@paginate(LimitOffsetPagination)
def list_runs(request):
    return Run.objects.all().order_by("-created_at")

@api.get("/runs/{run_id}", response=RunSchema)
def get_run(request, run_id: str):
    return Run.objects.get(id=run_id)

@api.get("/runs/{run_id}/steps", response=List[RunStepSchema])
@paginate(LimitOffsetPagination)
def list_run_steps(request, run_id: str):
    return RunStep.objects.filter(run_id=run_id)

@api.get("/graphs", response=List[GraphSchema])
@paginate(LimitOffsetPagination)
def list_graphs(request):
    return Graph.objects.all().order_by("-updated_at")

@api.get("/tenants", response=List[TenantSchema])
@paginate(LimitOffsetPagination)
def list_tenants(request):
    return Tenant.objects.all()

@api.get("/mcp/tools", response=List[Dict[str, Any]])
def list_mcp_tools(request):
    """List all generative tools managed by Django MCP."""
    return mcp_service.list_tools()

@api.post("/mcp/call/{tool_name}")
def call_mcp_tool(request, tool_name: str, params: Dict[str, Any]):
    """Execute an MCP tool via the standard Django MCP bridge."""
    return mcp_service.execute(tool_name, params)
