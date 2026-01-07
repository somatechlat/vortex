# VORTEX Deployment Guide

## ⚠️ DEPLOYMENT STACK: Colima + Minikube + Tilt (NO DOCKER DESKTOP)

This project uses a completely isolated Kubernetes development environment:
- **Colima** - Docker runtime (NOT Docker Desktop)
- **Minikube** - Local Kubernetes cluster
- **Tilt** - Development workflow orchestration

**DO NOT use Docker Desktop or Docker Compose directly.**

---

## Prerequisites

```bash
# Install Colima (Docker runtime for macOS)
brew install colima

# Install Minikube
brew install minikube

# Install Tilt
brew install tilt

# Install kubectl
brew install kubectl
```

---

## Quick Start

### 1. Start Colima
```bash
colima start
```

### 2. Create Isolated 'vortex' Minikube Profile
```bash
minikube start -p vortex \
  --disk-size=20g \
  --memory=10240 \
  --cpus=4 \
  --driver=docker
```

### 3. Deploy with Tilt
```bash
cd /path/to/vortex/gemini
tilt up
```

### 4. Access Dashboard
Open Tilt dashboard at `http://localhost:10350`

---

## Resource Limits (FIXED)

| Resource | Request | Limit |
|----------|---------|-------|
| Memory | 6Gi | 8Gi |
| CPU | 2 cores | 4 cores |
| Storage | 10Gi | - |

> **Note**: These limits ensure the cluster stays under 10GB total RAM.

---

## Resilient Startup Architecture

### Tier-Based Dependency Ordering

```
Tier 0: postgres, vault         ← No dependencies (root services)
   ↓
Tier 1: milvus, keycloak       ← Depend on Postgres
   ↓
Tier 2: spicedb                ← Depends on Postgres + Keycloak
   ↓
Tier 3: vortex-core            ← Depends on all above
   ↓
Tier 4: vortex-worker          ← Depends on vortex-core
   ↓
Tier 5: vortex-ui              ← Depends on vortex-core
```

### Init Containers

All services use init containers to wait for dependencies:

| Service | Waits For |
|---------|-----------|
| keycloak | postgres (port 5432) |
| vortex-core | postgres (5432), spicedb (50051) |
| vortex-worker | vortex-core (11188) |

### Recovery Time

| Phase | Duration |
|-------|----------|
| Tier 0 (Postgres, Vault) | ~30s |
| Tier 1 (Milvus, Keycloak) | ~2.5 min |
| Full Stack Ready | ~3 min |

---

## Cluster Isolation

The Tiltfile enforces isolation:
```python
allow_k8s_contexts('vortex')
```

This prevents accidental deployment to other clusters or profiles.

---

## Common Commands

```bash
# Check cluster status
kubectl --context=vortex get pods -n vortex

# View logs for a service
kubectl --context=vortex logs -n vortex -l app.kubernetes.io/name=vortex-core

# Restart all services
kubectl --context=vortex delete pods -n vortex --all

# Full teardown
tilt down
minikube stop -p vortex

# Delete cluster completely
minikube delete -p vortex
```

---

## Verified Resilience

Tested 7 consecutive pod delete/recreate cycles:
- **Result**: All 5/5 infrastructure pods recover to 1/1 Ready
- **Recovery Time**: ~3 minutes for full stack
- **Init Containers**: Working correctly (verified "Waiting for postgres...")
- **First-Time Migrations**: Keycloak auto-runs 117 DB migrations on fresh start

---

## Port Assignments

| Service | Port | Description |
|---------|------|-------------|
| vortex-core | 11188 | HTTP API |
| vortex-core | 11189 | WebSocket |
| vortex-core | 11191 | Metrics |
| vortex-ui | 11100 | Web UI |
| postgres | 11202 | Database |
| vault | 11200 | Secrets |
| keycloak | 11201 | Auth |
| milvus | 11203-11204 | Vector DB |
| spicedb | 11205-11206 | ReBAC |
