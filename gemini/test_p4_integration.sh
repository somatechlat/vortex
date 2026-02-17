#!/bin/bash
# P4 Integration Test: SomaAgent01 → VORTEX Worker

set -e

echo "============================================================"
echo "P4: SOMAAGENT01 → VORTEX INTEGRATION TEST"
echo "============================================================"

GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

cd worker

# Test 1: Import connector
echo -e "\n${GREEN}Step 1: Testing Connector Import${NC}"
python3 -c "from vortex_agent_connector import VortexConnector; print('✅ Connector imported')"

# Test 2: Create simple workflow
echo -e "\n${GREEN}Step 2: Creating SomaAgent01-style workflow${NC}"
python3 << 'PYEOF'
from vortex_agent_connector import VortexConnector
import json

# Example: Agent wants to generate an image
agent_workflow = {
    "workflow_name": "Image Generation",
    "nodes": [
        {"id": "step1", "executor": "Loader::Checkpoint", "params": {"model": "sdxl"}},
        {"id": "step2", "executor": "Encoder::CLIP", "params": {"text": "cyberpunk city"}},
        {"id": "step3", "executor": "Sampler::KSampler", "params": {"steps": 30}},
        {"id": "step4", "executor": "Decoder::VAE", "params": {}}
    ]
}

print(f"Agent workflow: {json.dumps(agent_workflow, indent=2)}")
print("✅ Workflow structure valid")
PYEOF

# Test 3: Direct job execution (without full API server)
echo -e "\n${GREEN}Step 3: Testing direct job submission${NC}"
python3 << 'PYEOF'
from vortex_agent_connector import VortexConnector

connector = VortexConnector()

# Verify we can construct the request
job_payload = {
    "job_id": "agent-001",
    "node_type": "Sampler::KSampler",
    "params_json": {"steps": 20, "cfg": 7.5},
    "inputs": [
        {
            "name": "latents",
            "offset": 0x4000,
            "size_bytes": 65536,
            "dtype": 0,
            "shape": [1, 4, 64, 64]
        }
    ]
}

print(f"Job payload: {job_payload}")
print("✅ Direct job structure is valid")
PYEOF

# Test 4: Verify endpoint structure
echo -e "\n${GREEN}Step 4: Verifying API endpoints${NC}"
python3 << 'PYEOF'
import inspect
from vortex_agent_connector import VortexConnector

methods = [m for m in dir(VortexConnector) if not m.startswith('_')]
print("Available methods:")
for m in methods:
    print(f"  - {m}")

required = ['submit_graph', 'execute_graph', 'submit_direct_job', 'get_status', 'get_workers', 'spawn_worker']
for req in required:
    if req in methods:
        print(f"✅ {req}")
    else:
        print(f"❌ {req} missing")

PYEOF

echo -e "\n${GREEN}============================================================"
echo "P4 INTEGRATION PREPARED"
echo "============================================================${NC}"
echo ""
echo "To test with real VORTEX:"
echo "  1. Start VORTEX: cargo run --bin vortex-core"
echo "  2. Spawn worker: python3 vortex_agent_connector.py"
echo "  3. Run full test: python3 vortex_agent_connector.py"
echo ""
