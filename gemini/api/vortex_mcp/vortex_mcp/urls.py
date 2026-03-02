"""
VORTEX MCP Server URLs
"""
from django.urls import path
from django.http import JsonResponse

def health_check(request):
    return JsonResponse({"status": "healthy", "service": "vortex-mcp"})

urlpatterns = [
    path("health", health_check),
]
