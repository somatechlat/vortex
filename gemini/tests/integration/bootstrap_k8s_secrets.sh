#!/usr/bin/env bash
set -euo pipefail

ok() { printf "✅ %s\n" "$1"; }
warn() { printf "⚠️  %s\n" "$1" >&2; }
fail() { printf "❌ %s\n" "$1" >&2; exit 1; }
note() { printf "ℹ️  %s\n" "$1" >&2; }

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    fail "Missing required command: $1"
  fi
}

generate_secret() {
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -hex 32
  elif command -v uuidgen >/dev/null 2>&1; then
    uuidgen | tr -d '-'
  else
    date +%s%N
  fi
}

ensure_value_file() {
  local name="$1"
  local file="$2"
  if [[ ! -f "$file" ]]; then
    generate_secret > "$file"
    note "Generated secret file: $file"
  fi
  cat "$file"
}

require_cmd kubectl

namespace="${VORTEX_NAMESPACE:-vortex}"
context="${VORTEX_K8S_CONTEXT:-vortex}"
secrets_dir="${VORTEX_SECRETS_DIR:-$HOME/.config/vortex/secrets}"

mkdir -p "$secrets_dir"

kubectl --context "$context" get namespace "$namespace" >/dev/null 2>&1 || \
  kubectl --context "$context" create namespace "$namespace" >/dev/null

postgres_password="$(ensure_value_file postgres_password "$secrets_dir/postgres_password")"
keycloak_admin_password="$(ensure_value_file keycloak_admin_password "$secrets_dir/keycloak_admin_password")"
spicedb_key="$(ensure_value_file spicedb_preshared_key "$secrets_dir/spicedb_preshared_key")"
vault_dev_token="$(ensure_value_file vault_dev_token "$secrets_dir/vault_dev_token")"

kubectl --context "$context" -n "$namespace" create secret generic postgres-credentials \
  --from-literal=password="$postgres_password" \
  --dry-run=client -o yaml | kubectl --context "$context" -n "$namespace" apply -f - >/dev/null

kubectl --context "$context" -n "$namespace" create secret generic keycloak-admin \
  --from-literal=password="$keycloak_admin_password" \
  --dry-run=client -o yaml | kubectl --context "$context" -n "$namespace" apply -f - >/dev/null

kubectl --context "$context" -n "$namespace" create secret generic spicedb-preshared-key \
  --from-literal=key="$spicedb_key" \
  --dry-run=client -o yaml | kubectl --context "$context" -n "$namespace" apply -f - >/dev/null

kubectl --context "$context" -n "$namespace" create secret generic vault-dev-token \
  --from-literal=token="$vault_dev_token" \
  --dry-run=client -o yaml | kubectl --context "$context" -n "$namespace" apply -f - >/dev/null

ok "Kubernetes secrets ensured in namespace: $namespace"
