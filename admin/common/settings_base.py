"""
VORTEX Centralized Django Settings Base
=======================================

Single source of truth for ALL Django services in VORTEX.
Every Django app (admin, MCP server, worker serve) MUST inherit from this.

Architecture:
- Secrets: Vault-first → ENV fallback → HARD FAIL (never hardcoded defaults)
- Database: Zero-Bleed prefixed env vars (VORTEX_POSTGRES_*)
- Security: Production-hardened by default, DEBUG opt-in only

Per Vibe Coding Rules:
- Rule #1: No hardcoded secrets, no fake defaults
- Rule #7: Real servers only
- Rule #11: Centralized messages via admin.common.messages
"""

import os
import sys
import logging

logger = logging.getLogger(__name__)


# ═══════════════════════════════════════════════════════════════
#                    SECRET MANAGEMENT
# ═══════════════════════════════════════════════════════════════

def get_required_env(key: str) -> str:
    """Get a required environment variable or hard-fail.

    This enforces Rule #1: No hardcoded defaults for secrets.
    In production, ALL secrets come from Vault → injected as ENV by K8s.
    """
    value = os.environ.get(key)
    if not value:
        raise RuntimeError(
            f"FATAL: Required environment variable '{key}' is not set. "
            f"Set it via Vault/K8s or export it before starting."
        )
    return value


def get_secret_key(service_name: str) -> str:
    """Get Django SECRET_KEY with Vault-first pattern.

    Priority:
    1. VORTEX_{SERVICE}_SECRET_KEY env var (injected by Vault/K8s)
    2. VORTEX_DJANGO_SECRET_KEY (shared fallback)
    3. HARD FAIL — never a hardcoded string
    """
    key = os.environ.get(f"VORTEX_{service_name}_SECRET_KEY")
    if key:
        return key

    key = os.environ.get("VORTEX_DJANGO_SECRET_KEY")
    if key:
        return key

    # In CI/test environments, generate a random key
    if os.environ.get("CI") or os.environ.get("VORTEX_TEST_MODE"):
        import secrets
        logger.warning("No SECRET_KEY configured; generating ephemeral key for CI/test")
        return secrets.token_urlsafe(50)

    raise RuntimeError(
        f"FATAL: No SECRET_KEY for service '{service_name}'. "
        f"Set VORTEX_{service_name}_SECRET_KEY or VORTEX_DJANGO_SECRET_KEY."
    )


# ═══════════════════════════════════════════════════════════════
#                    DATABASE CONFIG
# ═══════════════════════════════════════════════════════════════

def get_database_config() -> dict:
    """Centralized database configuration.

    Uses Zero-Bleed prefixed env vars (VORTEX_POSTGRES_*).
    Falls back to POSTGRES_* for backward compat.
    Password MUST be set — no empty-string fallback.
    """
    password = os.environ.get("VORTEX_POSTGRES_PASSWORD",
                               os.environ.get("POSTGRES_PASSWORD"))
    if not password:
        if os.environ.get("CI") or os.environ.get("VORTEX_TEST_MODE"):
            password = "test_password"
            logger.warning("No POSTGRES_PASSWORD set; using test default for CI")
        else:
            raise RuntimeError(
                "FATAL: Database password not configured. "
                "Set VORTEX_POSTGRES_PASSWORD or POSTGRES_PASSWORD."
            )

    return {
        "default": {
            "ENGINE": "django.db.backends.postgresql",
            "NAME": os.environ.get("VORTEX_POSTGRES_DB",
                                    os.environ.get("POSTGRES_DB", "vortex")),
            "USER": os.environ.get("VORTEX_POSTGRES_USER",
                                    os.environ.get("POSTGRES_USER", "vortex")),
            "PASSWORD": password,
            "HOST": os.environ.get("VORTEX_POSTGRES_HOST",
                                    os.environ.get("POSTGRES_HOST", "localhost")),
            "PORT": os.environ.get("VORTEX_POSTGRES_PORT",
                                    os.environ.get("POSTGRES_PORT", "5432")),
            "OPTIONS": {
                "options": "-c search_path=public"
            },
        }
    }


# ═══════════════════════════════════════════════════════════════
#                    SECURITY DEFAULTS
# ═══════════════════════════════════════════════════════════════

def get_allowed_hosts() -> list:
    """Get ALLOWED_HOSTS from environment.

    Never ["*"] in production. Only K8s service names and configured hosts.
    """
    hosts_str = os.environ.get("VORTEX_ALLOWED_HOSTS", "")
    if hosts_str:
        return [h.strip() for h in hosts_str.split(",") if h.strip()]

    # Safe defaults for development
    return ["localhost", "127.0.0.1", "::1"]


DEBUG = os.environ.get("VORTEX_DEBUG", "False").lower() in ("true", "1")


# ═══════════════════════════════════════════════════════════════
#                    SHARED MIDDLEWARE
# ═══════════════════════════════════════════════════════════════

# Full middleware stack for apps with sessions/auth
FULL_MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "django.contrib.sessions.middleware.SessionMiddleware",
    "django.middleware.common.CommonMiddleware",
    "django.middleware.csrf.CsrfViewMiddleware",
    "django.contrib.auth.middleware.AuthenticationMiddleware",
    "django.contrib.messages.middleware.MessageMiddleware",
    "django.middleware.clickjacking.XFrameOptionsMiddleware",
]

# Lightweight middleware for API-only services (no sessions, no CSRF)
API_MIDDLEWARE = [
    "django.middleware.security.SecurityMiddleware",
    "django.middleware.common.CommonMiddleware",
]


# ═══════════════════════════════════════════════════════════════
#                    SECURITY HARDENING
# ═══════════════════════════════════════════════════════════════

def apply_security_hardening(settings_dict: dict) -> None:
    """Apply production security settings when DEBUG is False."""
    if not settings_dict.get("DEBUG", False):
        settings_dict["SECURE_SSL_REDIRECT"] = True
        settings_dict["SESSION_COOKIE_SECURE"] = True
        settings_dict["CSRF_COOKIE_SECURE"] = True
        settings_dict["SECURE_BROWSER_XSS_FILTER"] = True
        settings_dict["SECURE_CONTENT_TYPE_NOSNIFF"] = True
        settings_dict["X_FRAME_OPTIONS"] = "DENY"


# ═══════════════════════════════════════════════════════════════
#                    LOGGING CONFIG
# ═══════════════════════════════════════════════════════════════

def get_logging_config(service_name: str) -> dict:
    """Centralized logging config for all Django services."""
    log_level = os.environ.get("VORTEX_LOG_LEVEL", "INFO").upper()
    return {
        "version": 1,
        "disable_existing_loggers": False,
        "formatters": {
            "verbose": {
                "format": f"[VORTEX {service_name.upper()}] {{levelname}} {{asctime}} {{module}} {{message}}",
                "style": "{",
            },
        },
        "handlers": {
            "console": {
                "class": "logging.StreamHandler",
                "formatter": "verbose",
            },
        },
        "root": {
            "handlers": ["console"],
            "level": log_level,
        },
    }


# Shared constants
LANGUAGE_CODE = "en-us"
TIME_ZONE = "UTC"
USE_I18N = True
USE_TZ = True
DEFAULT_AUTO_FIELD = "django.db.models.BigAutoField"
