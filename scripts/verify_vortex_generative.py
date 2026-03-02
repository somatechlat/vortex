import os
import django
import json
import logging

# 1. Setup Django Environment
os.environ.setdefault("DJANGO_SETTINGS_MODULE", "vortex_admin.settings")
os.environ["VORTEX_POSTGRES_DB"] = "vortex" # Zero-Bleed prefix
os.environ["DATABASE_URL"] = "postgres://vortex:@localhost:5432/vortex"

# Add the admin and gemini/api paths to sys.path
import sys
sys.path.append(os.path.abspath("admin"))
sys.path.append(os.path.abspath("gemini/api/django_admin"))

try:
    django.setup()
    from vortex_core.mcp import mcp_service
    from vortex_core.api import router
except Exception as e:
    print(f"FAILED TO LOAD VORTEX STACK: {e}")
    sys.exit(1)

# 2. Register Generative Tools (Simulating Worker discovery)
print("--- [VORTEX] Registering Tools in Django MCP Bridge ---")
mcp_service.register_tool("image.flux", {
    "name": "image.flux",
    "description": "High-fidelity festive image generation",
    "input_schema": {"prompt": "string"}
})

mcp_service.register_tool("video.svd", {
    "name": "video.svd",
    "description": "Christmas-themed video interpolation",
    "input_schema": {"image_ref": "string", "motion_bucket": "int"}
})

# 3. Simulate API Call for "Christmas Flying Cats"
print("\n--- [VORTEX] Orchestrating Image Generation ---")
image_result = mcp_service.execute("image.flux", {
    "prompt": "Stunning cinematic cats flying in Christmas sky"
})
print(f"Result: {json.dumps(image_result, indent=2)}")

print("\n--- [VORTEX] Orchestrating Video Generation ---")
video_result = mcp_service.execute("video.svd", {
    "image_ref": "shm_offset_0x1234",
    "motion_bucket": 127
})
print(f"Result: {json.dumps(video_result, indent=2)}")
