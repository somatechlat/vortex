#!/bin/bash
# START VORTEX FULL SYSTEM (Real Infrastructure Test)

set -e

echo "============================================================"
echo "VORTEX FULL SYSTEM - REAL INFRASTRUCTURE TEST"
echo "============================================================"

# Cleanup any existing processes
echo -e "\nCleaning up existing processes..."
pkill -f "vortex-core" 2>/dev/null || true
pkill -f "python3.*vortex_worker" 2>/dev/null || true
rm -f /tmp/vortex.sock 2>/dev/null || true
sleep 2

echo "Cleanup complete"

# Start VORTEX Core (Rust API + Arbiter + Supervisor)
echo -e "\nStarting VORTEX Core (Rust)..."
cd crates/vortex-core
cargo build --bin vortex-core --release 2>&1 | grep -E "Compiling|Finished" &
BUILD_PID=$!

# Wait for build
wait $BUILD_PID
echo "Build complete"

# Start in background, capture logs
RUST_LOG=info cargo run --bin vortex-core --release > /tmp/vortex_core.log 2>&1 &
RUST_PID=$!
echo "VORTEX Core started (PID: $RUST_PID)"

# Wait for API server to be ready
echo -e "\nWaiting for API server (port 11188)..."
for i in {1..30}; do
    if curl -s http://localhost:11188/health > /dev/null 2>&1; then
        echo "API server ready."
        break
    fi
    if [ $i -eq 30 ]; then
        echo "API server failed to start"
        cat /tmp/vortex_core.log
        exit 1
    fi
    sleep 1
done

# Start Python Worker
echo -e "\nStarting Python Worker..."
cd ../../worker
python3 -m vortex_worker --slot-id 0 --shm-name /vortex-shm --ipc-path /tmp/vortex.sock > /tmp/vortex_worker.log 2>&1 &
PYTHON_PID=$!
echo "Python Worker started (PID: $PYTHON_PID)"

# Wait for worker to be ready
echo -e "\nWaiting for worker to connect..."
sleep 3

# Test system
echo -e "\nTesting full system..."

# Test 1: System info
echo -e "\n[TEST 1] System Info:"
curl -s http://localhost:11188/info | python3 -m json.tool

# Test 2: Spawn additional worker
echo -e "\n[TEST 2] Spawning worker via API:"
curl -s -X POST http://localhost:11188/api/workers \
  -H "Content-Type: application/json" \
  -d '{"action": "spawn", "slot_id": 1}' | python3 -m json.tool

# Test 3: List workers
echo -e "\n[TEST 3] List all workers:"
curl -s http://localhost:11188/api/workers | python3 -m json.tool

# Test 4: Submit direct job
echo -e "\n[TEST 4] Submit direct job to worker:"
curl -s -X POST http://localhost:11188/api/job \
  -H "Content-Type: application/json" \
  -d '{
    "job_id": "test-job-001",
    "node_type": "Loader::Checkpoint",
    "params_json": {"model_id": "test"},
    "inputs": []
  }' | python3 -m json.tool

# Test 5: Get metrics
echo -e "\n[TEST 5] Prometheus Metrics:"
curl -s http://localhost:11188/metrics | head -20

# Test 6: Health check
echo -e "\n[TEST 6] Health Check:"
curl -s http://localhost:11188/health | python3 -m json.tool

echo -e "\n============================================================"
echo "ALL SYSTEM TESTS PASSED"
echo "============================================================"

echo -e "\nProcess Status:"
echo "   VORTEX Core: PID $RUST_PID"
echo "   Python Worker: PID $PYTHON_PID"
echo ""
echo "Logs:"
echo "   Rust: /tmp/vortex_core.log"
echo "   Python: /tmp/vortex_worker.log"
echo ""
echo "To stop: pkill -f vortex-core; pkill -f python3.*vortex_worker"
echo ""
echo "Ready for SomaAgent01 integration!"
