//! Lockfile (vortex.lock) Parser/Writer

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// Lockfile structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    pub version: u32,
    pub packages: HashMap<String, LockedPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    pub version: String,
    pub hash: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl Lockfile {
    pub fn new() -> Self {
        Self {
            version: 1,
            packages: HashMap::new(),
        }
    }
    
    /// Parse lockfile from TOML
    pub fn parse(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }
    
    /// Serialize to TOML
    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
    
    /// Add a package to the lockfile
    pub fn add_package(&mut self, name: String, version: String, content_hash: &[u8]) {
        let hash = hex::encode(content_hash);
        self.packages.insert(name, LockedPackage {
            version,
            hash,
            dependencies: Vec::new(),
        });
    }
    
    /// Verify package hash
    pub fn verify(&self, name: &str, content: &[u8]) -> bool {
        if let Some(pkg) = self.packages.get(name) {
            let mut hasher = Sha256::new();
            hasher.update(content);
            let hash = hex::encode(hasher.finalize());
            hash == pkg.hash
        } else {
            false
        }
    }
}

impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}

// Need hex for hash encoding
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}
