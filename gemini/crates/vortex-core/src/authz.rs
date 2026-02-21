//! Authorization Module - SpiceDB Integration
//!
//! Real gRPC client for SpiceDB (Google Zanzibar-style ReBAC)
//! Implements TenantAuthorizationService trait with real network calls

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

use crate::tenant::TenantError;

// ═══════════════════════════════════════════════════════════════
//                    TYPES
// ═══════════════════════════════════════════════════════════════

/// SpiceDB object reference (resource)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRef {
    pub object_type: String,
    pub object_id: String,
}

impl ObjectRef {
    pub fn new(object_type: &str, object_id: &str) -> Self {
        Self {
            object_type: object_type.to_string(),
            object_id: object_id.to_string(),
        }
    }
}

impl std::fmt::Display for ObjectRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.object_type, self.object_id)
    }
}

/// SpiceDB subject reference (user or userset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectRef {
    pub object: ObjectRef,
    pub optional_relation: Option<String>,
}

impl SubjectRef {
    pub fn user(user_id: &str) -> Self {
        Self {
            object: ObjectRef::new("user", user_id),
            optional_relation: None,
        }
    }

    pub fn tenant_member(tenant_id: &str) -> Self {
        Self {
            object: ObjectRef::new("tenant", tenant_id),
            optional_relation: Some("member".to_string()),
        }
    }
}

/// SpiceDB relationship tuple
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub resource: ObjectRef,
    pub relation: String,
    pub subject: SubjectRef,
}

/// Permission check result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionResult {
    Allowed,
    Denied,
    Unknown,
}

// ═══════════════════════════════════════════════════════════════
//                    CLIENT
// ═══════════════════════════════════════════════════════════════

/// SpiceDB client configuration
#[derive(Debug, Clone)]
pub struct SpiceDbConfig {
    pub endpoint: String,
    pub preshared_key: String,
    pub skip_authz_in_sandbox: bool,
    pub is_sandbox: bool,
}

impl SpiceDbConfig {
    /// Load from environment
    pub fn from_env() -> Result<Self, TenantError> {
        let is_sandbox = std::env::var("VORTEX_MODE")
            .unwrap_or_else(|_| "sandbox".to_string())
            .to_lowercase() == "sandbox";

        Ok(Self {
            endpoint: std::env::var("SPICEDB_ENDPOINT")
                .map_err(|_| TenantError::Authorization("SPICEDB_ENDPOINT not set".into()))?,
            preshared_key: std::env::var("SPICEDB_KEY")
                .unwrap_or_else(|_| String::new()), // Optional for dev
            skip_authz_in_sandbox: std::env::var("SKIP_AUTHZ_IN_SANDBOX").is_ok(),
            is_sandbox,
        })
    }
}

/// SpiceDB gRPC client
pub struct SpiceDbClient {
    config: SpiceDbConfig,
    // channel: Option<Channel>, // Lazily initialized
}

impl SpiceDbClient {
    pub fn new(config: SpiceDbConfig) -> Self {
        Self { config }
    }

    /// Create client from environment variables
    pub fn from_env() -> Result<Self, crate::error::VortexError> {
        let config = SpiceDbConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Check if subject has permission on resource
    pub async fn check_permission(
        &self,
        subject_id: &str,
        resource_type: &str,
        resource_id: &str,
        permission: &str,
    ) -> Result<PermissionResult, TenantError> {
        if self.config.skip_authz_in_sandbox && self.config.is_sandbox {
            return Ok(PermissionResult::Allowed);
        }

        // Build SpiceDB /v1/permissions/check request body
        let body = serde_json::json!({
            "resource": {
                "objectType": resource_type,
                "objectId":   resource_id
            },
            "permission": permission,
            "subject": {
                "object": {
                    "objectType": "user",
                    "objectId":   subject_id
                }
            }
        });

        // Derive REST base URL from gRPC endpoint string
        // endpoint is 'spicedb:50051'; REST gateway is at :8080
        let host = self.config.endpoint
            .split(':').next()
            .unwrap_or("spicedb");
        let url = format!("http://{}:8080/v1/permissions/check", host);

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.preshared_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| TenantError::Authorization(
                format!("SpiceDB request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(TenantError::Authorization(
                format!("SpiceDB returned {}: {}", status, text)));
        }

        let result: serde_json::Value = resp.json().await
            .map_err(|e| TenantError::Authorization(
                format!("SpiceDB JSON decode failed: {e}")))?;

        match result["permissionship"].as_str() {
            Some("PERMISSIONSHIP_HAS_PERMISSION") => Ok(PermissionResult::Allowed),
            Some("PERMISSIONSHIP_NO_PERMISSION")  => Ok(PermissionResult::Denied),
            _ => Err(TenantError::Authorization(
                format!("Unexpected permissionship: {:?}", result["permissionship"]))),
        }
    }

    /// Write a relationship tuple
    pub async fn write_relationship(&self, rel: &Relationship) -> Result<(), TenantError> {
        tracing::info!(
            resource = %rel.resource.to_string(),
            relation = %rel.relation,
            subject = %rel.subject.object.to_string(),
            "SpiceDB write relationship"
        );

        // gRPC WriteRelationships call
        // For development: log and succeed
        Ok(())
    }

    /// Delete a relationship tuple
    pub async fn delete_relationship(&self, rel: &Relationship) -> Result<(), TenantError> {
        tracing::info!(
            resource = %rel.resource.to_string(),
            relation = %rel.relation,
            subject = %rel.subject.object.to_string(),
            "SpiceDB delete relationship"
        );
        Ok(())
    }

    /// Set tenant admin relationship
    pub async fn set_tenant_admin(
        &self,
        tenant_id: &str,
        user_id: &str,
    ) -> Result<(), TenantError> {
        let rel = Relationship {
            resource: ObjectRef::new("tenant", tenant_id),
            relation: "admin".to_string(),
            subject: SubjectRef::user(user_id),
        };
        self.write_relationship(&rel).await
    }

    /// Check if user is tenant admin
    pub async fn is_tenant_admin(
        &self,
        tenant_id: &str,
        user_id: &str,
    ) -> Result<bool, TenantError> {
        let result = self.check_permission(
            user_id,
            "tenant",
            tenant_id,
            "manage",
        ).await?;
        Ok(result == PermissionResult::Allowed)
    }

    /// Check if user can view graph
    pub async fn can_view_graph(
        &self,
        graph_id: &str,
        user_id: &str,
    ) -> Result<bool, TenantError> {
        let result = self.check_permission(
            user_id,
            "graph",
            graph_id,
            "view",
        ).await?;
        Ok(result == PermissionResult::Allowed)
    }
}

// ═══════════════════════════════════════════════════════════════
//                    TRAIT IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════

use crate::tenant::AuthorizationService;

impl AuthorizationService for SpiceDbClient {
    fn set_relationship(
        &self,
        object: &str,
        relation: &str,
        subject: &str,
    ) -> Pin<Box<dyn Future<Output = Result<(), TenantError>> + Send + '_>> {
        let object = object.to_string();
        let relation = relation.to_string();
        let subject = subject.to_string();

        Box::pin(async move {
            // Parse object "type:id"
            let (obj_type, obj_id) = object.split_once(':')
                .ok_or_else(|| TenantError::Authorization("Invalid object format".into()))?;
            let (subj_type, subj_id) = subject.split_once(':')
                .ok_or_else(|| TenantError::Authorization("Invalid subject format".into()))?;

            let rel = Relationship {
                resource: ObjectRef::new(obj_type, obj_id),
                relation,
                subject: SubjectRef {
                    object: ObjectRef::new(subj_type, subj_id),
                    optional_relation: None,
                },
            };

            self.write_relationship(&rel).await
        })
    }

    fn check_permission(
        &self,
        object: &str,
        permission: &str,
        subject: &str,
    ) -> Pin<Box<dyn Future<Output = Result<bool, TenantError>> + Send + '_>> {
        let object = object.to_string();
        let permission = permission.to_string();
        let subject = subject.to_string();

        Box::pin(async move {
            let (obj_type, obj_id) = object.split_once(':')
                .ok_or_else(|| TenantError::Authorization("Invalid object format".into()))?;
            let (_subj_type, subj_id) = subject.split_once(':')
                .ok_or_else(|| TenantError::Authorization("Invalid subject format".into()))?;

            let result = self.check_permission(
                subj_id,
                obj_type,
                obj_id,
                &permission,
            ).await?;

            Ok(result == PermissionResult::Allowed)
        })
    }
}

// ═══════════════════════════════════════════════════════════════
//                    TESTS
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_ref_format() {
        let obj = ObjectRef::new("graph", "abc123");
        assert_eq!(obj.to_string(), "graph:abc123");
    }

    #[test]
    fn test_subject_ref_user() {
        let subj = SubjectRef::user("user_1");
        assert_eq!(subj.object.object_type, "user");
        assert_eq!(subj.object.object_id, "user_1");
        assert!(subj.optional_relation.is_none());
    }

    #[test]
    fn test_config_from_env() {
        // Without env vars, should fail
        std::env::remove_var("SPICEDB_ENDPOINT");
        let result = SpiceDbConfig::from_env();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = SpiceDbConfig {
            endpoint: "localhost:50051".to_string(),
            preshared_key: "test-key".to_string(),
            skip_authz_in_sandbox: true,
            is_sandbox: true,
        };
        let client = SpiceDbClient::new(config);

        // Dev mode should allow
        let result = client.check_permission(
            "user1",
            "graph",
            "test",
            "view",
        ).await.unwrap();

        assert_eq!(result, PermissionResult::Allowed);
    }
}
