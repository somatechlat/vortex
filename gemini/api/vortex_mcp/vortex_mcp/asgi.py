"""
VORTEX MCP Server ASGI Configuration

Uses django-mcp to mount MCP server
"""
import os
import django
from django.core.asgi import get_asgi_application
from django_mcp import mount_mcp_server

os.environ.setdefault('DJANGO_SETTINGS_MODULE', 'vortex_mcp.settings')

django.setup()

django_http_app = get_asgi_application()

# Mount MCP server at /mcp endpoint
application = mount_mcp_server(
    django_http_app=django_http_app,
    mcp_base_path='/mcp'
)
