//! Virtual Environment Management
//!
//! Manages isolated Python environments for custom nodes.

use std::path::PathBuf;
use std::collections::HashMap;

/// Virtual environment info
#[derive(Clone, Debug)]
pub struct VenvInfo {
    pub name: String,
    pub path: PathBuf,
    pub python: PathBuf,
    pub packages: Vec<InstalledPackage>,
}

#[derive(Clone, Debug)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
}

/// Virtual environment manager
pub struct VenvManager {
    base_path: PathBuf,
    python_path: PathBuf,
    envs: HashMap<String, VenvInfo>,
}

impl VenvManager {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            python_path: PathBuf::from("python3"),
            envs: HashMap::new(),
        }
    }
    
    /// Create a new virtual environment
    pub async fn create(&mut self, name: &str) -> Result<VenvInfo, std::io::Error> {
        let env_path = self.base_path.join(name);
        
        // Run python -m venv
        let status = tokio::process::Command::new(&self.python_path)
            .args(["-m", "venv", env_path.to_str().unwrap()])
            .status()
            .await?;
        
        if !status.success() {
            return Err(std::io::Error::other("Failed to create venv"));
        }
        
        let info = VenvInfo {
            name: name.to_string(),
            path: env_path.clone(),
            python: env_path.join("bin/python"),
            packages: Vec::new(),
        };
        
        self.envs.insert(name.to_string(), info.clone());
        
        Ok(info)
    }
    
    /// Get an existing environment
    pub fn get(&self, name: &str) -> Option<&VenvInfo> {
        self.envs.get(name)
    }
    
    /// List all environments
    pub fn list(&self) -> Vec<&VenvInfo> {
        self.envs.values().collect()
    }
}
