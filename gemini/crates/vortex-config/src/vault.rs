//! HashiCorp Vault Client for VORTEX
//!
//! Handles secure retrieval of secrets at runtime.

use serde::Deserialize;
use crate::ConfigError;

pub struct VaultClient {
    addr: String,
    token: String,
    mount: String,
}

#[derive(Deserialize)]
struct VaultResponse {
    data: VaultData,
}

#[derive(Deserialize)]
struct VaultData {
    data: std::collections::HashMap<String, String>,
}

impl VaultClient {
    pub fn new(addr: String, token: String, mount: String) -> Self {
        Self { addr, token, mount }
    }

    /// Retrieve a secret from Vault
    pub async fn get_secret(&self, path: &str, key: &str) -> Result<String, ConfigError> {
        let url = format!("{}/v1/{}/data/{}", self.addr, self.mount, path);

        let client = reqwest::Client::new();
        let resp = client.get(&url)
            .header("X-Vault-Token", &self.token)
            .send()
            .await
            .map_err(|e| ConfigError::Io(format!("Vault connection failed: {}", e)))?;

        if !resp.status().is_success() {
            return Err(ConfigError::Validation(format!(
                "Vault returned error: {} for path {}", resp.status(), path
            )));
        }

        let body: VaultResponse = resp.json()
            .await
            .map_err(|e| ConfigError::Parse(format!("Failed to parse Vault response: {}", e)))?;

        body.data.data.get(key)
            .cloned()
            .ok_or_else(|| ConfigError::MissingField(format!("Key {} not found in Vault path {}", key, path)))
    }
}
