use crate::error::{Result, SmooothError};
use crate::models::agent::AgentScope;
use serde::{Deserialize, Serialize};

/// Multi-user Team Role-Based Access Control classifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamRole {
    /// Read-only access to inspection, logs, and monitoring
    Viewer,
    /// Authoring plans and deploying to development/staging environments
    Developer,
    /// Production operations, reconciliation, ingress, and routine approvals
    Operator,
    /// Unrestricted security administration, destructive permissions, and master operations
    Admin,
}

impl TeamRole {
    /// Maps the team role to the corresponding platform AgentScopes
    pub fn granted_scopes(&self) -> Vec<AgentScope> {
        match self {
            TeamRole::Viewer => vec![AgentScope::ReadOnly],
            TeamRole::Developer => vec![AgentScope::ReadOnly, AgentScope::Plan, AgentScope::Deploy],
            TeamRole::Operator => vec![AgentScope::ReadOnly, AgentScope::Plan, AgentScope::Deploy],
            TeamRole::Admin => vec![
                AgentScope::ReadOnly,
                AgentScope::Plan,
                AgentScope::Deploy,
                AgentScope::Destructive,
                AgentScope::Admin,
            ],
        }
    }

    /// Validates whether this role is authorized to deploy into a specific target environment
    pub fn can_deploy_to_environment(&self, environment: &str) -> bool {
        let is_prod = environment.eq_ignore_ascii_case("production")
            || environment.eq_ignore_ascii_case("prod");

        match self {
            TeamRole::Viewer => false,
            TeamRole::Developer => !is_prod, // Developers cannot deploy directly to production
            TeamRole::Operator => true,
            TeamRole::Admin => true,
        }
    }

    /// Validates whether this role can approve pending execution plans
    pub fn can_approve_plans(&self) -> bool {
        matches!(self, TeamRole::Operator | TeamRole::Admin)
    }

    /// Enforces environmental deployment authorization, returning an explicit error if denied
    pub fn authorize_environment_deployment(&self, environment: &str) -> Result<()> {
        if self.can_deploy_to_environment(environment) {
            Ok(())
        } else {
            Err(SmooothError::PolicyViolation(format!(
                "RBAC authorization failure: Role '{:?}' is not permitted to deploy to '{}' environment. Operator or Admin role required.",
                self, environment
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewer_role_boundaries() {
        let viewer = TeamRole::Viewer;
        assert_eq!(viewer.granted_scopes(), vec![AgentScope::ReadOnly]);
        assert!(!viewer.can_deploy_to_environment("development"));
        assert!(!viewer.can_deploy_to_environment("production"));
        assert!(!viewer.can_approve_plans());
    }

    #[test]
    fn test_developer_role_environment_guard() {
        let dev = TeamRole::Developer;
        assert!(dev.can_deploy_to_environment("development"));
        assert!(dev.can_deploy_to_environment("staging"));
        assert!(!dev.can_deploy_to_environment("production"));
        assert!(dev.authorize_environment_deployment("production").is_err());
    }

    #[test]
    fn test_operator_and_admin_privileges() {
        let op = TeamRole::Operator;
        assert!(op.can_deploy_to_environment("production"));
        assert!(op.can_approve_plans());

        let admin = TeamRole::Admin;
        assert!(admin.can_deploy_to_environment("production"));
        assert!(admin.granted_scopes().contains(&AgentScope::Destructive));
        assert!(admin.granted_scopes().contains(&AgentScope::Admin));
    }
}
