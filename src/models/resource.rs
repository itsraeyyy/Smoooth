use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceStatus {
    Provisioning,
    Active,
    Degraded,
    Stopping,
    Stopped,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub workspace_id: String,
    pub service_type: String,
    pub provider: String,
    pub is_persistent: bool,
    #[serde(default)]
    pub ownership: crate::models::ResourceOwnership,
    pub status: ResourceStatus,
    pub configuration: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Resource {
    /// Retrieve organization ID, defaulting to "default"
    pub fn org_id(&self) -> &str {
        self.configuration
            .get("org_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
    }

    /// Retrieve project ID, defaulting to "default"
    pub fn project_id(&self) -> &str {
        self.configuration
            .get("project_id")
            .and_then(|v| v.as_str())
            .unwrap_or("default")
    }

    /// Retrieve environment, defaulting to "production"
    pub fn environment(&self) -> &str {
        self.configuration
            .get("environment")
            .and_then(|v| v.as_str())
            .unwrap_or("production")
    }

    /// Retrieve optional host server ID
    pub fn server_id(&self) -> Option<&str> {
        self.configuration.get("server_id").and_then(|v| v.as_str())
    }

    /// Construct structured TenancyContext representation
    pub fn tenancy(&self) -> crate::models::tenancy::TenancyContext {
        crate::models::tenancy::TenancyContext {
            org_id: self.org_id().to_string(),
            project_id: self.project_id().to_string(),
            environment: self.environment().to_string(),
            workspace_id: self.workspace_id.clone(),
            server_id: self.server_id().map(|s| s.to_string()),
        }
    }
}
