"""Secret access helpers with no heavy runtime dependencies."""

from __future__ import annotations

import logging
import os
from typing import Any

import hvac

logger = logging.getLogger(__name__)


def get_vault_secret(path: str, key: str, default: Any = None) -> Any:
    """Fetch a secret from HashiCorp Vault and return `default` on failure."""
    vault_addr = os.getenv("VAULT_ADDR", "http://127.0.0.1:8200")
    vault_token = os.getenv("VAULT_TOKEN")

    if not vault_token:
        logger.warning("VAULT_TOKEN not found; using configured default.")
        return default

    try:
        client = hvac.Client(url=vault_addr, token=vault_token)
        if not client.is_authenticated():
            logger.error("Vault authentication failed.")
            return default

        response = client.secrets.kv.v2.read_secret_version(
            mount_point="secret",
            path=path,
        )
        return response["data"]["data"].get(key, default)
    except Exception as exc:
        logger.error("Failed to fetch secret from Vault: %s", exc)
        return default
