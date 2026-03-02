"""
VORTEX MCP Server Settings

Django 5 + django-mcp implementation.
Inherits all security and DB config from admin.common.settings_base.

Per VIBE Coding Rules: All Python APIs MUST use Django.
"""

import os
import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent

# Add project root so 'admin' package is importable
PROJECT_ROOT = BASE_DIR.parent.parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

# ═══════════════════════════════════════════════════════════════
#                    CENTRALIZED BASE
# ═══════════════════════════════════════════════════════════════
from admin.common.settings_base import (
    get_secret_key,
    get_database_config,
    get_allowed_hosts,
    get_logging_config,
    API_MIDDLEWARE,
    LANGUAGE_CODE,
    TIME_ZONE,
    USE_I18N,
    USE_TZ,
    DEBUG,
)

# ═══════════════════════════════════════════════════════════════
#                    SERVICE IDENTITY
# ═══════════════════════════════════════════════════════════════
SERVICE_NAME = "MCP"

SECRET_KEY = get_secret_key(SERVICE_NAME)
ALLOWED_HOSTS = get_allowed_hosts()
DATABASES = get_database_config()

# ═══════════════════════════════════════════════════════════════
#                    APPLICATIONS
# ═══════════════════════════════════════════════════════════════
INSTALLED_APPS = [
    "django.contrib.contenttypes",
    "django.contrib.auth",
    "django_mcp",  # django-mcp package
    "vortex_mcp.tools",  # Our MCP tools
]

# API-only service: lightweight middleware (no sessions, no CSRF needed for
# machine-to-machine MCP protocol communication over SSE/WebSocket)
MIDDLEWARE = API_MIDDLEWARE

ROOT_URLCONF = "vortex_mcp.urls"

# VORTEX Core API Connection
VORTEX_CORE_API_URL = os.environ.get("VORTEX_CORE_API_URL", "http://localhost:11188")

# ═══════════════════════════════════════════════════════════════
#                    django-mcp Configuration
# ═══════════════════════════════════════════════════════════════
MCP_LOG_LEVEL = "INFO"
MCP_LOG_TOOL_REGISTRATION = True
MCP_LOG_TOOL_DESCRIPTIONS = True
MCP_SERVER_INSTRUCTIONS = (
    "VORTEX AI Workflow Engine - Execute generative AI workflows "
    "with human-in-the-loop approval gates"
)
MCP_SERVER_TITLE = "VORTEX MCP Server"
MCP_SERVER_VERSION = "1.0.0"

# ═══════════════════════════════════════════════════════════════
#                    LOGGING
# ═══════════════════════════════════════════════════════════════
LOGGING = get_logging_config(SERVICE_NAME)
