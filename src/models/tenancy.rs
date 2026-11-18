use crate::error::{Result, SmooothError};
use serde::{Deserialize, Serialize};

pub fn default_org_id() -> String {
    "default".to_string()
}

pub fn default_project_id() -> String {
    "default".to_string()
}

pub fn default_environment() -> String {
    "production".to_string()
}

pub fn default_workspace_id() -> String {
    "default".to_string()
}

/// Validates organization, project, and environment identifiers against directory traversal and control characters
pub fn validate_tenancy_identifier(kind: &str, id: &str) -> Result<()> {
    let trimmed = id.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return Err(SmooothError::Validation(format!(
            "{} identifier must be between 1 and 64 characters",
            kind
        )));
    }
    let valid = trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !valid
        || trimmed.starts_with('-')
        || trimmed.ends_with('-')
        || trimmed.starts_with('_')
        || trimmed.ends_with('_')
    {
        return Err(SmooothError::Validation(format!(
            "{} identifier '{}' contains invalid characters. Must be alphanumeric with dashes or underscores.",
            kind, trimmed
        )));
    }
    Ok(())
}

/// First-class tenancy context for multi-organization, multi-project, multi-environment deployments.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TenancyContext {
    #[serde(default = "default_org_id")]
    pub org_id: String,
    #[serde(default = "default_project_id")]
    pub project_id: String,
    #[serde(default = "default_environment")]
    pub environment: String,
    #[serde(default = "default_workspace_id")]
    pub workspace_id: String,
    #[serde(default)]
    pub server_id: Option<String>,
}

impl TenancyContext {
    pub fn new(
        org_id: &str,
        project_id: &str,
        environment: &str,
        workspace_id: &str,
    ) -> Result<Self> {
        validate_tenancy_identifier("Organization", org_id)?;
        validate_tenancy_identifier("Project", project_id)?;
        validate_tenancy_identifier("Environment", environment)?;
        validate_tenancy_identifier("Workspace", workspace_id)?;

        Ok(Self {
            org_id: org_id.to_string(),
            project_id: project_id.to_string(),
            environment: environment.to_string(),
            workspace_id: workspace_id.to_string(),
            server_id: None,
        })
    }

    pub fn with_server(mut self, server_id: &str) -> Result<Self> {
        validate_tenancy_identifier("Server", server_id)?;
        self.server_id = Some(server_id.to_string());
        Ok(self)
    }

    pub fn is_production(&self) -> bool {
        self.environment.eq_ignore_ascii_case("production")
            || self.environment.eq_ignore_ascii_case("prod")
    }

    pub fn canonical_hierarchy(&self) -> String {
        format!(
            "orgs/{}/projects/{}/environments/{}/workspaces/{}",
            self.org_id, self.project_id, self.environment, self.workspace_id
        )
    }
}

impl Default for TenancyContext {
    fn default() -> Self {
        Self {
            org_id: default_org_id(),
            project_id: default_project_id(),
            environment: default_environment(),
            workspace_id: default_workspace_id(),
            server_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_tenancy_context() {
        let t = TenancyContext::new("my-org", "my-project", "production", "primary").unwrap();
        assert!(t.is_production());
        assert_eq!(
            t.canonical_hierarchy(),
            "orgs/my-org/projects/my-project/environments/production/workspaces/primary"
        );
    }

    #[test]
    fn test_invalid_tenancy_identifier_rejected() {
        assert!(TenancyContext::new("my..org", "proj", "prod", "ws").is_err());
        assert!(TenancyContext::new("org", "proj/slash", "prod", "ws").is_err());
        assert!(TenancyContext::new("-bad-leading", "proj", "prod", "ws").is_err());
    }
}
