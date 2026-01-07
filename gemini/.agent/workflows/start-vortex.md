---
description: Start the VORTEX development environment with Tilt + Minikube
---

# VORTEX Development Environment Startup

This workflow starts the complete VORTEX stack on Minikube with live code updates via Tilt.

## Prerequisites

- Minikube installed
- Tilt installed
- Rust toolchain (for vortex-core)
- Bun (for vortex-ui)
- Python 3.12 (for vortex-worker)

## Steps

// turbo-all

1. Ensure Minikube is running with the vortex profile:
```bash
minikube start -p vortex --driver=vfkit --memory 7500 --cpus 4
```

2. Set kubectl context:
```bash
kubectl config use-context vortex
```

3. Configure Docker to use Minikube's internal registry:
```bash
eval $(minikube docker-env -p vortex)
```

4. Start Tilt with live updates:
```bash
cd /Users/macbookpro201916i964gb1tb/Documents/GitHub/Vortex/gemini && tilt up
```

5. Wait for infrastructure pods to be ready (Postgres, Vault, Milvus, Keycloak, SpiceDB):
```bash
kubectl wait --for=condition=ready pod -l app=postgres -n vortex --timeout=300s
kubectl wait --for=condition=ready pod -l app=vault -n vortex --timeout=300s
```

6. Open Tilt dashboard:
```bash
open http://localhost:10350/
```

## Live Editing

Once Tilt is running, edit any file in:
- `crates/` → Triggers Rust rebuild for vortex-core
- `worker/` → Syncs Python code to vortex-worker
- `ui/src/` → Triggers Svelte rebuild for vortex-ui

## Port Forwards

| Service | Local Port | Internal Port |
|:--------|:-----------|:--------------|
| Tilt Dashboard | 10350 | - |
| Vault | 11200 | 8200 |
| Keycloak | 11201 | 8080 |
| Postgres | 11202 | 5432 |
| Milvus | 11203 | 19530 |
| SpiceDB | 11205 | 50051 |
| vortex-core | 11188 | 11188 |
| vortex-ui | 11100 | 80 |

## Shutdown

```bash
tilt down
minikube stop -p vortex
```
