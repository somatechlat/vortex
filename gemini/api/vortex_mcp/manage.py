#!/usr/bin/env python3
"""
VORTEX MCP Server - Django Management Script

MCP (Model Context Protocol) server using django-mcp package
"""
import os
import sys

if __name__ == "__main__":
    os.environ.setdefault("DJANGO_SETTINGS_MODULE", "vortex_mcp.settings")
    
    try:
        from django.core.management import execute_from_command_line
    except ImportError as exc:
        raise ImportError(
            "Couldn't import Django. Are you sure it's installed?"
        ) from exc
    
    execute_from_command_line(sys.argv)
