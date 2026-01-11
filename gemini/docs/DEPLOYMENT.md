# VORTEX Deployment Guide

## ⚙️ Deployment Methods (Isolated)

This project provides **isolated** deployment paths. Each method is self-contained and does not mix with the others.

**Available methods:**
1. **Docker Compose (Standalone)** — `gemini/infra/docker/standalone/`
2. **Kubernetes + Tilt (Minikube)** — `gemini/infra/tilt/`

---

## Method 1: Docker Compose (Standalone)

**Scope:** Local, single-host deployment (no Kubernetes).  
**Folder:** `gemini/infra/docker/standalone/`

### Prerequisites
- Docker Engine (Docker Desktop or Colima)

### Quick Start
```bash
cd /path/to/vortex/gemini/infra/docker/standalone
docker compose up -d --build
```

### Secrets (Isolated)
Secrets live in `gemini/infra/docker/standalone/secrets/` and are **not committed**.  
Regenerate them if needed:
```bash
openssl rand -hex 32 > gemini/infra/docker/standalone/secrets/postgres_password
openssl rand -hex 32 > gemini/infra/docker/standalone/secrets/keycloak_admin_password
openssl rand -hex 32 > gemini/infra/docker/standalone/secrets/spicedb_preshared_key
openssl rand -hex 32 > gemini/infra/docker/standalone/secrets/vault_dev_token
```

### Ports
| Service | Port | Description |
|---------|------|-------------|
| vortex-core | 11188 | HTTP API |
| vortex-core | 11189 | WebSocket |
| vortex-core | 11191 | Metrics |
| vortex-ui | 11100 | Web UI |
| vault | 11200 | Secrets |
| keycloak | 11201 | Auth |
| postgres | 11202 | Database |
| milvus | 11203-11204 | Vector DB |
| spicedb | 11205-11206 | ReBAC |

---

## Method 2: Kubernetes + Tilt (Minikube)

**Scope:** Local Kubernetes cluster, isolated to the `vortex` context.  
**Folder:** `gemini/infra/tilt/`

### Prerequisites
```bash
# Docker runtime (Colima or Docker Desktop)
brew install colima
brew install minikube
brew install tilt
brew install kubectl
```

### Quick Start
```bash
colima start
minikube start -p vortex --disk-size=20g --memory=7800 --cpus=4 --driver=docker
cd /path/to/vortex/gemini/infra/tilt
tilt up
```

### Cluster Isolation
The Tiltfile enforces isolation:
```python
allow_k8s_contexts('vortex')
```

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
