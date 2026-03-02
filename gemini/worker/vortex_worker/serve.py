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
from typing import Any
import numpy as np
import torch
from django.conf import settings
from django.core.wsgi import get_wsgi_application
from django.urls import path
from django.http import HttpResponse

# Initialize VORTEX Components
from .shm import ShmArena
from .executor import ExecutorRegistry, TensorHandle
from .secrets import get_vault_secret
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
# Django Configuration (Single File)
# Uses centralized settings_base for secrets and security.
# -----------------------------------------------------------------------------

if not settings.configured:
    # Vault-first secret chain: Vault → ENV → CI ephemeral → HARD FAIL
    SECRET_KEY = get_vault_secret(
        "vortex/prod", "django_secret_key",
        default=os.getenv("VORTEX_WORKER_SECRET_KEY"),
    )
    if not SECRET_KEY:
        # Fallback to centralized secret key logic
        try:
            import sys as _sys
            _project_root = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
            if _project_root not in _sys.path:
                _sys.path.insert(0, _project_root)
            from admin.common.settings_base import get_secret_key, get_allowed_hosts, get_logging_config
            SECRET_KEY = get_secret_key("WORKER")
            _ALLOWED_HOSTS = get_allowed_hosts()
            _LOGGING = get_logging_config("WORKER")
        except (ImportError, RuntimeError):
            # Worker may run in isolated container without admin package
            # In that case, env var is mandatory
            SECRET_KEY = os.getenv("VORTEX_DJANGO_SECRET_KEY")
            if not SECRET_KEY:
                raise RuntimeError(
                    "FATAL: No SECRET_KEY for worker. Set VORTEX_WORKER_SECRET_KEY "
                    "or VORTEX_DJANGO_SECRET_KEY, or configure Vault."
                )
            _ALLOWED_HOSTS = [h.strip() for h in os.getenv("VORTEX_ALLOWED_HOSTS", "localhost,127.0.0.1,::1").split(",")]
            _LOGGING = {
                "version": 1,
                "disable_existing_loggers": False,
                "handlers": {"console": {"class": "logging.StreamHandler"}},
                "root": {"handlers": ["console"], "level": "INFO"},
            }
    else:
        _ALLOWED_HOSTS = [h.strip() for h in os.getenv("VORTEX_ALLOWED_HOSTS", "localhost,127.0.0.1,::1").split(",")]
        _LOGGING = {
            "version": 1,
            "disable_existing_loggers": False,
            "handlers": {"console": {"class": "logging.StreamHandler"}},
            "root": {"handlers": ["console"], "level": "INFO"},
        }

    settings.configure(
        DEBUG=False,
        SECRET_KEY=SECRET_KEY,
        ROOT_URLCONF=__name__,
        ALLOWED_HOSTS=_ALLOWED_HOSTS,
        INSTALLED_APPS=[
            "django.contrib.contenttypes",
            "ninja",
        ],
        LOGGING=_LOGGING,
    )

# -----------------------------------------------------------------------------
# Django Ninja API
# -----------------------------------------------------------------------------

from ninja import NinjaAPI, Schema

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
