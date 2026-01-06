# VORTEX Environment Variables & Configuration Catalog
## Complete Settings and Configuration Reference

> **Standard**: 12-Factor App  
> **Version**: 1.0.0  
> **Status**: PLANNING

---

## 1. ENVIRONMENT VARIABLE CATALOG

### 1.1 Core Engine Variables

| Variable | Type | Default | Required | Description |
|----------|------|---------|----------|-------------|
| `VORTEX_ENV` | enum | `development` | ❌ | Environment: `development`, `staging`, `production` |
| `VORTEX_PORT` | int | `11188` | ❌ | HTTP API port (Port Authority: 11000+) |
| `VORTEX_WS_PORT` | int | `11189` | ❌ | WebSocket port |
| `VORTEX_HOST` | string | `0.0.0.0` | ❌ | Bind address |
| `VORTEX_WORKERS` | int | `4` | ❌ | Number of Python worker processes |
| `VORTEX_LOG_LEVEL` | enum | `info` | ❌ | Log level: `error`, `warn`, `info`, `debug`, `trace` |
| `VORTEX_LOG_FORMAT` | enum | `json` | ❌ | Log format: `json`, `pretty` |

### 1.2 Shared Memory Variables

| Variable | Type | Default | Required | Description |
|----------|------|---------|----------|-------------|
| `VORTEX_SHM_NAME` | string | `vtx_arena` | ❌ | POSIX shared memory name |
| `VORTEX_SHM_SIZE` | size | `68719476736` | ❌ | Arena size in bytes (default: 64GB) |
| `VORTEX_SHM_PATH` | path | `/dev/shm` | ❌ | SHM mount path |

### 1.3 IPC Variables

| Variable | Type | Default | Required | Description |
|----------|------|---------|----------|-------------|
| `VORTEX_SOCKET_PATH` | path | `/tmp/vtx.sock` | ❌ | Unix domain socket path |
| `VORTEX_SOCKET_TIMEOUT` | int | `30000` | ❌ | Socket timeout in ms |
| `VORTEX_HEARTBEAT_INTERVAL` | int | `1000` | ❌ | Worker heartbeat interval in ms |

### 1.4 Database Variables

| Variable | Type | Default | Required | Description |
|----------|------|---------|----------|-------------|
| `VORTEX_DB_PATH` | path | `./vortex.db` | ❌ | SQLite database path |
| `VORTEX_DB_WAL` | bool | `true` | ❌ | Enable WAL mode |
| `VORTEX_DB_POOL_SIZE` | int | `5` | ❌ | Connection pool size |

### 1.5 GPU / VRAM Variables

| Variable | Type | Default | Required | Description |
|----------|------|---------|----------|-------------|
| `VORTEX_VRAM_LIMIT` | size | `0` | ❌ | VRAM limit (0 = auto-detect) |
| `VORTEX_GPU_DEVICE` | string | `cuda:0` | ❌ | Default GPU device |
| `VORTEX_CPU_ONLY` | bool | `false` | ❌ | Force CPU-only mode |
| `VORTEX_PRECISION` | enum | `fp16` | ❌ | Default precision: `fp16`, `fp32`, `bf16` |

### 1.6 Security Variables

| Variable | Type | Default | Required | Description |
|----------|------|---------|----------|-------------|
| `VORTEX_SANDBOX_ENABLED` | bool | `true` | ❌ | Enable worker sandboxing |
| `VORTEX_SECCOMP_POLICY` | path | (built-in) | ❌ | Custom Seccomp policy path |
| `VORTEX_ALLOW_NETWORK` | bool | `false` | ❌ | Allow worker network access |
| `VORTEX_SCAN_NODES` | bool | `true` | ❌ | Scan custom nodes for malware |

### 1.7 Telemetry Variables

| Variable | Type | Default | Required | Description |
|----------|------|---------|----------|-------------|
| `VORTEX_METRICS_PORT` | int | `11191` | ❌ | Prometheus metrics port |
| `VORTEX_METRICS_ENABLED` | bool | `true` | ❌ | Enable metrics collection |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | url | - | ❌ | OpenTelemetry collector endpoint |
| `OTEL_SERVICE_NAME` | string | `vortex-core` | ❌ | Service name for traces |

### 1.8 UI Variables

| Variable | Type | Default | Required | Description |
|----------|------|---------|----------|-------------|
| `VORTEX_UI_PORT` | int | `11173` | ❌ | Frontend dev server port |
| `VORTEX_API_URL` | url | `http://localhost:11188` | ❌ | Backend API URL |
| `VORTEX_WS_URL` | url | `ws://localhost:11189` | ❌ | WebSocket URL |
| `VORTEX_THEME` | enum | `dark` | ❌ | Default theme: `dark`, `light` |

### 1.9 Paths & Directories

| Variable | Type | Default | Required | Description |
|----------|------|---------|----------|-------------|
| `VORTEX_DATA_DIR` | path | `~/.vortex` | ❌ | Data directory |
| `VORTEX_MODELS_DIR` | path | `~/.vortex/models` | ❌ | Models directory |
| `VORTEX_OUTPUTS_DIR` | path | `~/.vortex/outputs` | ❌ | Generated outputs |
| `VORTEX_CACHE_DIR` | path | `~/.vortex/cache` | ❌ | Cache directory |
| `VORTEX_CUSTOM_NODES` | path | `~/.vortex/custom_nodes` | ❌ | Custom nodes path |

---

## 2. CONFIGURATION FILE REFERENCE

### 2.1 Main Config: `vortex.toml`

```toml
# vortex.toml - Main configuration file

[server]
host = "0.0.0.0"
port = 11188
ws_port = 11189
workers = 4

[shm]
name = "vtx_arena"
size = "64GB"

[database]
path = "./vortex.db"
wal = true
pool_size = 5

[gpu]
device = "cuda:0"
vram_limit = "0"          # 0 = auto
precision = "fp16"

[security]
sandbox = true
scan_nodes = true
allow_network = false

[telemetry]
metrics_port = 11191
log_level = "info"
log_format = "json"

[paths]
data = "~/.vortex"
models = "~/.vortex/models"
outputs = "~/.vortex/outputs"
custom_nodes = "~/.vortex/custom_nodes"
```

### 2.2 Environment File: `.env.example`

```bash
# .env.example - Environment template

# ========================================
# CORE ENGINE
# ========================================
VORTEX_ENV=development
VORTEX_PORT=11188
VORTEX_WS_PORT=11189
VORTEX_HOST=0.0.0.0
VORTEX_WORKERS=4

# ========================================
# LOGGING
# ========================================
VORTEX_LOG_LEVEL=info
VORTEX_LOG_FORMAT=json
RUST_LOG=vortex_core=debug

# ========================================
# SHARED MEMORY
# ========================================
VORTEX_SHM_NAME=vtx_arena
VORTEX_SHM_SIZE=68719476736

# ========================================
# IPC
# ========================================
VORTEX_SOCKET_PATH=/tmp/vtx.sock
VORTEX_HEARTBEAT_INTERVAL=1000

# ========================================
# DATABASE
# ========================================
VORTEX_DB_PATH=./vortex.db
VORTEX_DB_WAL=true

# ========================================
# GPU / VRAM
# ========================================
VORTEX_GPU_DEVICE=cuda:0
VORTEX_VRAM_LIMIT=0
VORTEX_PRECISION=fp16
VORTEX_CPU_ONLY=false

# ========================================
# SECURITY
# ========================================
VORTEX_SANDBOX_ENABLED=true
VORTEX_ALLOW_NETWORK=false
VORTEX_SCAN_NODES=true

# ========================================
# TELEMETRY
# ========================================
VORTEX_METRICS_PORT=11191
VORTEX_METRICS_ENABLED=true
# OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# ========================================
# PATHS
# ========================================
VORTEX_DATA_DIR=~/.vortex
VORTEX_MODELS_DIR=~/.vortex/models
VORTEX_OUTPUTS_DIR=~/.vortex/outputs
VORTEX_CACHE_DIR=~/.vortex/cache
VORTEX_CUSTOM_NODES=~/.vortex/custom_nodes

# ========================================
# UI (Development Only)
# ========================================
VORTEX_UI_PORT=11173
VORTEX_API_URL=http://localhost:11188
VORTEX_WS_URL=ws://localhost:11189
```

---

## 3. SETTINGS CATEGORIES

### 3.1 User-Configurable Settings (UI)

| Category | Setting | Type | Default | Storage |
|----------|---------|------|---------|---------|
| **Appearance** | Theme | enum | `dark` | localStorage |
| **Appearance** | Accent Color | color | `#58a6ff` | localStorage |
| **Appearance** | Font Size | enum | `medium` | localStorage |
| **Appearance** | Grid Visible | bool | `true` | localStorage |
| **Appearance** | Minimap Visible | bool | `true` | localStorage |
| **Performance** | WebGL Enabled | bool | `true` | localStorage |
| **Performance** | LOD Threshold | float | `0.6` | localStorage |
| **Performance** | Max Undo Steps | int | `100` | localStorage |
| **Workflow** | Auto-Save | bool | `true` | localStorage |
| **Workflow** | Auto-Save Interval | int | `30` | localStorage |
| **Workflow** | Confirm Delete | bool | `true` | localStorage |
| **Shortcuts** | Custom Bindings | map | (defaults) | localStorage |

### 3.2 System Settings (Server)

| Category | Setting | Type | Default | Storage |
|----------|---------|------|---------|---------|
| **Execution** | Queue Limit | int | `100` | Database |
| **Execution** | Timeout | int | `3600` | Database |
| **Execution** | Retry Count | int | `3` | Database |
| **Models** | Default Checkpoint | string | - | Database |
| **Models** | Default VAE | string | - | Database |
| **Cache** | Max Size | size | `10GB` | Config |
| **Cache** | TTL | int | `86400` | Config |

---

## 4. ENVIRONMENT PROFILES

### 4.1 Development Profile

| Variable | Value | Rationale |
|----------|-------|-----------|
| `VORTEX_ENV` | `development` | Enable dev features |
| `VORTEX_LOG_LEVEL` | `debug` | Verbose logging |
| `VORTEX_LOG_FORMAT` | `pretty` | Human-readable logs |
| `VORTEX_SANDBOX_ENABLED` | `false` | Easier debugging |
| `VORTEX_WORKERS` | `2` | Reduced resource usage |

### 4.2 Staging Profile

| Variable | Value | Rationale |
|----------|-------|-----------|
| `VORTEX_ENV` | `staging` | Near-production |
| `VORTEX_LOG_LEVEL` | `info` | Normal logging |
| `VORTEX_LOG_FORMAT` | `json` | Structured logs |
| `VORTEX_SANDBOX_ENABLED` | `true` | Security enabled |

### 4.3 Production Profile

| Variable | Value | Rationale |
|----------|-------|-----------|
| `VORTEX_ENV` | `production` | Production mode |
| `VORTEX_LOG_LEVEL` | `warn` | Minimal logging |
| `VORTEX_LOG_FORMAT` | `json` | Log aggregation |
| `VORTEX_SANDBOX_ENABLED` | `true` | Security enforced |
| `VORTEX_METRICS_ENABLED` | `true` | Monitoring |

---

## 5. VARIABLE PRECEDENCE

Variables are loaded in this order (later overrides earlier):

```
1. Built-in defaults (compiled into binary)
        ↓
2. System config (/etc/vortex/vortex.toml)
        ↓
3. User config (~/.config/vortex/vortex.toml)
        ↓
4. Project config (./vortex.toml)
        ↓
5. Environment file (.env)
        ↓
6. Environment variables (OS)
        ↓
7. Command-line arguments
```

---

## 6. VALIDATION RULES

| Variable | Validation | Error |
|----------|------------|-------|
| `VORTEX_PORT` | 1024-65535, != common ports | `VE-101: Invalid port` |
| `VORTEX_SHM_SIZE` | ≥ 1GB, ≤ 256GB | `VE-102: SHM size out of range` |
| `VORTEX_WORKERS` | 1-256 | `VE-103: Invalid worker count` |
| `VORTEX_LOG_LEVEL` | Valid enum | `VE-104: Unknown log level` |
| `VORTEX_GPU_DEVICE` | Valid CUDA device | `VE-105: GPU not found` |

---

## 7. SECRETS MANAGEMENT

| Secret | Storage | Access |
|--------|---------|--------|
| API Keys | Environment / Vault | `VORTEX_*_API_KEY` |
| Private Keys | Filesystem (0600) | `VORTEX_*_KEY_PATH` |
| Tokens | Environment | `VORTEX_*_TOKEN` |

**Never commit secrets to version control.**

---

**Document Status**: COMPLETE  
**Total Variables**: 45+  
**Total Settings**: 20+  
**Ready for Implementation**: ✅
