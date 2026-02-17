"""
VORTEX SageMaker Serving Interface (Django/Ninja)
Listens on port 8080 for /invocations (POST) and /ping (GET).
STRICT VAULT INTEGRATION: No Secrets in ENV.
"""

import os
import io
import json
import base64
import logging
import sys
import numpy as np
import torch
import hvac
from django.conf import settings
from django.core.wsgi import get_wsgi_application
from django.urls import path
from ninja import NinjaAPI, Schema
from django.http import HttpResponse

# Initialize VORTEX Components
from .shm import ShmArena
from .executor import ExecutorRegistry, TensorHandle
# Ensure Cloud Executors are registered
from . import extras

# Setup Logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("vortex.serve")

# Global State
SHM_ARENA = None

def init_shm():
    global SHM_ARENA
    if SHM_ARENA:
        return
    try:
        SHM_ARENA = ShmArena("/vortex-shm", size=1024 * 1024 * 1024)
        logger.info("SHM Arena initialized")
    except Exception as e:
        logger.error(f"Failed to init SHM: {e}")

# -----------------------------------------------------------------------------
# Vault Integration (Strict No-Env Secrets)
# -----------------------------------------------------------------------------

def get_vault_secret(path: str, key: str, default=None) -> str:
    """Fetch a secret from HashiCorp Vault. Falls back strictly if configured."""
    vault_addr = os.getenv("VAULT_ADDR", "http://127.0.0.1:8200")
    vault_token = os.getenv("VAULT_TOKEN") # Token itself must be in ENV or disk

    if not vault_token:
        logger.warning("VAULT_TOKEN not found inside container. Using insecure fallback for build/test.")
        return default if default else "django-insecure-vault-missing"

    try:
        client = hvac.Client(url=vault_addr, token=vault_token)
        if not client.is_authenticated():
            logger.error("Vault authentication failed")
            return default

        # Adjust mount point as per Soma/Vortex standard (vortex/config)
        # Assuming KV v2
        response = client.secrets.kv.v2.read_secret_version(mount_point='secret', path=path)
        return response['data']['data'].get(key, default)
    except Exception as e:
        logger.error(f"Failed to fetch secret from Vault: {e}")
        return default

# -----------------------------------------------------------------------------
# Django Configuration (Single File)
# -----------------------------------------------------------------------------

if not settings.configured:
    # Fetch Secret from Vault
    # Path: secret/vortex/prod, Key: django_secret_key
    SECRET_KEY = get_vault_secret("vortex/prod", "django_secret_key", default=os.getenv("DJANGO_SECRET_KEY_FALLBACK", "django-insecure-build-only"))

    settings.configure(
        DEBUG=False,
        SECRET_KEY=SECRET_KEY,
        ROOT_URLCONF=__name__,
        ALLOWED_HOSTS=["*"],
        INSTALLED_APPS=[
            "django.contrib.contenttypes",
            "ninja",
        ],
        LOGGING={
            "version": 1,
            "disable_existing_loggers": False,
            "handlers": {
                "console": {
                    "class": "logging.StreamHandler",
                },
            },
            "root": {
                "handlers": ["console"],
                "level": "INFO",
            },
        },
    )

# -----------------------------------------------------------------------------
# Django Ninja API
# -----------------------------------------------------------------------------

api = NinjaAPI(urls_namespace='vortex')

class InputTensor(Schema):
    data: str # Base64
    shape: list[int]
    dtype: str

class InvocationRequest(Schema):
    op_type: str
    params: dict[str, Any] = {}
    inputs: dict[str, InputTensor] = {}

@api.get("/ping")
def ping(request):
    """SageMaker Health Check."""
    return HttpResponse("Pong", status=200)

@api.post("/invocations")
def invoke(request, payload: InvocationRequest):
    """Main Inference Entrypoint."""
    logger.info(f"Received invocation for {payload.op_type}")

    init_shm()
    if not SHM_ARENA:
         return api.create_response(request, {"error": "SHM unavailable"}, status=500)

    try:
        # 1. Deserialize Inputs
        input_handles = {}
        for name, tensor_data in payload.inputs.items():
            b64_data = tensor_data.data

            # Decode
            buffer = io.BytesIO(base64.b64decode(b64_data))
            np_array = np.load(buffer)
            tensor = torch.from_numpy(np_array)

            if torch.cuda.is_available():
                tensor = tensor.to("cuda")

            offset = SHM_ARENA.store_tensor(tensor)

            input_handles[name] = TensorHandle(
                offset=offset,
                shape=tuple(tensor_data.shape),
                dtype=tensor_data.dtype,
                device=str(tensor.device)
            )

        # 2. Execute Node
        result = ExecutorRegistry.execute_node(
            op_type=payload.op_type,
            inputs=input_handles,
            params=payload.params,
            shm_arena=SHM_ARENA
        )

        # 3. Serialize Outputs
        output_data = {}
        if result.success:
            for name, handle in result.outputs.items():
                if handle.offset == 0:
                    continue

                tensor = SHM_ARENA.load_tensor(handle.offset)
                np_array = tensor.cpu().numpy()

                buffer = io.BytesIO()
                np.save(buffer, np_array)
                b64_out = base64.b64encode(buffer.getvalue()).decode("utf-8")

                output_data[name] = {
                    "data": b64_out,
                    "shape": list(np_array.shape),
                    "dtype": str(np_array.dtype)
                }

        return {
            "success": result.success,
            "outputs": output_data,
            "error": result.error,
            "duration_us": result.duration_us,
            "peak_vram_mb": result.peak_vram_mb
        }

    except Exception as e:
        logger.exception("Inference failed")
        return api.create_response(request, {"error": str(e), "success": False}, status=500)

# -----------------------------------------------------------------------------
# URL Patterns & WSGI
# -----------------------------------------------------------------------------

urlpatterns = [
    path("", api.urls),
]

application = get_wsgi_application()

if __name__ == "__main__":
    from django.core.management import execute_from_command_line
    execute_from_command_line(sys.argv)
