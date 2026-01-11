#!/usr/bin/env bash
set -euo pipefail

ok() { printf "✅ %s\n" "$1"; }
warn() { printf "⚠️  %s\n" "$1"; }
fail() { printf "❌ %s\n" "$1"; exit 1; }

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    fail "Missing required command: $1"
  fi
}

check_http() {
  local name="$1"
  local url="$2"
  if curl -s -f "$url" >/dev/null; then
    ok "$name reachable: $url"
  else
    fail "$name not reachable: $url"
  fi
}

check_postgres() {
  local host="${POSTGRES_HOST:-localhost}"
  local port="${POSTGRES_PORT:-5432}"
  local db="${POSTGRES_DB:-vortex}"
  local user="${POSTGRES_USER:-vortex}"
  local password_file="${VORTEX_DB_PASSWORD_PATH:-$HOME/.config/vortex/secrets/postgres_password}"

  if ! command -v pg_isready >/dev/null 2>&1; then
    warn "pg_isready not found; skipping PostgreSQL readiness check"
    return
  fi

  if [[ -f "$password_file" ]]; then
    export PGPASSWORD
    PGPASSWORD="$(cat "$password_file")"
  elif [[ -n "${POSTGRES_PASSWORD:-}" ]]; then
    warn "POSTGRES_PASSWORD is set in env; prefer file-based secret"
    export PGPASSWORD="${POSTGRES_PASSWORD}"
  else
    warn "No postgres password provided; readiness check may fail if auth required"
  fi

  if pg_isready -h "$host" -p "$port" -d "$db" -U "$user" >/dev/null 2>&1; then
    ok "PostgreSQL ready at ${host}:${port}/${db}"
  else
    fail "PostgreSQL not ready at ${host}:${port}/${db}"
  fi
}

check_spicedb() {
  local endpoint="${SPICEDB_ENDPOINT:-}"
  if [[ -z "$endpoint" ]]; then
    warn "SPICEDB_ENDPOINT not set; skipping SpiceDB check"
    return
  fi
  if ! command -v grpc_health_probe >/dev/null 2>&1; then
    warn "grpc_health_probe not found; skipping SpiceDB check"
    return
  fi
  if grpc_health_probe -addr="$endpoint" >/dev/null 2>&1; then
    ok "SpiceDB healthy at $endpoint"
  else
    fail "SpiceDB not healthy at $endpoint"
  fi
}

require_cmd curl
check_postgres

check_http "Core API" "${VORTEX_API_URL:-http://localhost:11188/health}"
check_http "Vault" "${VAULT_ADDR:-http://localhost:11200}/v1/sys/health"
check_http "Keycloak" "http://localhost:11201/health/ready"

check_spicedb

ok "Preflight checks complete"
