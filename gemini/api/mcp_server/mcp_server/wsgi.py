"""
WSGI config for VORTEX MCP Server

Django 5 + Django Ninja per VIBE Coding Rules
"""
import os
from django.core.wsgi import get_wsgi_application

os.environ.setdefault("DJANGO_SETTINGS_MODULE", "mcp_server.settings")

application = get_wsgi_application()
