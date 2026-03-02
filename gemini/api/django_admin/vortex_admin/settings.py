"""
VORTEX Admin Settings - Django Integration Layer

Read-only admin monitoring for the Rust execution engine.
Inherits all security and DB config from admin.common.settings_base.

CRITICAL: Never write to DB from Django - all writes happen via Rust API.
Django is read-only for monitoring and user management only.
"""

import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent

# Add project root so 'admin' package is importable
PROJECT_ROOT = BASE_DIR.parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

# ═══════════════════════════════════════════════════════════════
#                    CENTRALIZED BASE
# ═══════════════════════════════════════════════════════════════
from admin.common.settings_base import (
    get_secret_key,
    get_database_config,
    get_allowed_hosts,
    apply_security_hardening,
    get_logging_config,
    FULL_MIDDLEWARE,
    LANGUAGE_CODE,
    TIME_ZONE,
    USE_I18N,
    USE_TZ,
    DEFAULT_AUTO_FIELD,
    DEBUG,
)

# ═══════════════════════════════════════════════════════════════
#                    SERVICE IDENTITY
# ═══════════════════════════════════════════════════════════════
SERVICE_NAME = "ADMIN"

SECRET_KEY = get_secret_key(SERVICE_NAME)
ALLOWED_HOSTS = get_allowed_hosts()
DATABASES = get_database_config()

# Read-only safety
DATABASES["default"]["TEST"] = {"NAME": "test_vortex"}

# ═══════════════════════════════════════════════════════════════
#                    APPLICATIONS
# ═══════════════════════════════════════════════════════════════
INSTALLED_APPS = [
    "django.contrib.admin",
    "django.contrib.auth",
    "django.contrib.contenttypes",
    "django.contrib.sessions",
    "django.contrib.messages",
    "django.contrib.staticfiles",
    # VORTEX apps
    "admin.vortex_core",
]

MIDDLEWARE = FULL_MIDDLEWARE

ROOT_URLCONF = "vortex_admin.urls"

TEMPLATES = [
    {
        "BACKEND": "django.template.backends.django.DjangoTemplates",
        "DIRS": [],
        "APP_DIRS": True,
        "OPTIONS": {
            "context_processors": [
                "django.template.context_processors.request",
                "django.contrib.auth.context_processors.auth",
                "django.contrib.messages.context_processors.messages",
            ],
        },
    },
]

# ═══════════════════════════════════════════════════════════════
#                    SECURITY / LOGGING / STATIC
# ═══════════════════════════════════════════════════════════════
STATIC_URL = "/static/"
STATIC_ROOT = BASE_DIR / "staticfiles"

LOGGING = get_logging_config(SERVICE_NAME)

# Apply production hardening (SSL, secure cookies, etc.)
_settings = {k: v for k, v in locals().items() if k.isupper()}
apply_security_hardening(_settings)
for k, v in _settings.items():
    if k not in locals() or locals()[k] != v:
        globals()[k] = v
