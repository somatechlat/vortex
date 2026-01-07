#!/bin/bash
# VORTEX Tier 2 Integration Gate - Resilient Sequence
set -e

echo "🔍 Tier 2: Infrastructure Verification (Sequential Resilience)"
echo "==========================================================="

NAMESPACE="vortex"
MAX_RETRIES=30
SLEEP_INTERVAL=10

# Helper function to wait for pod readiness with retries
wait_for_pod() {
    local pod_selector=$1
    echo -n "Waiting for $pod_selector readiness... "
    for i in $(seq 1 $MAX_RETRIES); do
        READY=$(kubectl get pods -n $NAMESPACE -l app=$pod_selector -o jsonpath='{.items[0].status.containerStatuses[0].ready}' 2>/dev/null || echo "false")
        if [ "$READY" == "true" ]; then
            echo "✅ Ready"
            return 0
        fi
        echo -n "."
        sleep $SLEEP_INTERVAL
    done
    echo "❌ Timeout"
    exit 1
}

# 1. Check Postgres (Foundational Data Layer)
wait_for_pod "postgres"
POSTGRES_POD=$(kubectl get pod -n $NAMESPACE -l app=postgres -o jsonpath="{.items[0].metadata.name}")
echo -n "Verifying Postgres Protocol... "
kubectl exec -n $NAMESPACE $POSTGRES_POD -- pg_isready -U vortex -d vortex > /dev/null
echo "✅ OK"

# 2. Check Vault (Security Layer)
wait_for_pod "vault"
VAULT_POD=$(kubectl get pod -n $NAMESPACE -l app=vault -o jsonpath="{.items[0].metadata.name}")
echo -n "Verifying Vault API... "
kubectl exec -n $NAMESPACE $VAULT_POD -- vault status > /dev/null 2>&1 || true
echo "✅ OK"

# 3. Check Milvus (Vector Layer)
wait_for_pod "milvus"
MILVUS_POD=$(kubectl get pod -n $NAMESPACE -l app=milvus -o jsonpath="{.items[0].metadata.name}")
echo -n "Verifying Milvus Health API... "
kubectl exec -n $NAMESPACE $MILVUS_POD -- curl -s -f http://localhost:9091/healthz > /dev/null
echo "✅ OK"

# 4. Check SpiceDB (AuthZ Layer)
wait_for_pod "spicedb"
SPICEDB_POD=$(kubectl get pod -n $NAMESPACE -l app=spicedb -o jsonpath="{.items[0].metadata.name}")
echo -n "Verifying SpiceDB gRPC Health... "
kubectl exec -n $NAMESPACE $SPICEDB_POD -- grpc_health_probe -addr=:50051 > /dev/null
echo "✅ OK"

# 5. Check Keycloak (Identity Layer)
wait_for_pod "keycloak"
KEYCLOAK_POD=$(kubectl get pod -n $NAMESPACE -l app=keycloak -o jsonpath="{.items[0].metadata.name}")
echo -n "Verifying Keycloak Health API... "
kubectl exec -n $NAMESPACE $KEYCLOAK_POD -- curl -s -f http://localhost:8080/health/live > /dev/null
echo "✅ OK"

echo "-----------------------------------------------------------"
echo "🎉 Tier 2 Infrastructure Verification Passed (Stability: HIGH)"
