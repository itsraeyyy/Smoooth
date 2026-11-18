use crate::error::{Result, SmooothError};
use serde::{Deserialize, Serialize};

/// Commercial tier classifications matching the Smoooth Strategic & Product Blueprint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommercialPlan {
    /// Free adoption engine: Self-hosted single-node daemon, core MCP/CLI, basic operations
    Community,
    /// $9–15/mo: Managed Smoooth Cloud control plane, multiple environments, agent credentials
    Developer,
    /// $29–49/mo: Multi-agent, advanced capability policies, production approval workflows
    Pro,
    /// $79–149/mo: Team members, RBAC, centralized multi-server control, extended audit retention
    Team,
    /// Custom: SSO/SAML, private deployment, compliance packs, dedicated SLA
    Business,
}

impl Default for CommercialPlan {
    fn default() -> Self {
        Self::Community
    }
}

/// Feature capabilities gated by commercial plan entitlements
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeatureGate {
    MultiWorkspace,
    RemoteCloudControlPlane,
    ProductionApprovalWorkflow,
    TeamRoleBasedAccess,
    CentralizedMultiServerManagement,
    ExtendedAuditRetention,
}

pub struct EntitlementEngine {
    plan: CommercialPlan,
}

impl EntitlementEngine {
    pub fn new(plan: CommercialPlan) -> Self {
        Self { plan }
    }

    pub fn current_plan(&self) -> CommercialPlan {
        self.plan
    }

    /// Determines whether the active subscription allows the requested feature
    pub fn is_feature_allowed(&self, feature: &FeatureGate) -> bool {
        match feature {
            FeatureGate::MultiWorkspace => self.plan >= CommercialPlan::Developer,
            FeatureGate::RemoteCloudControlPlane => self.plan >= CommercialPlan::Developer,
            FeatureGate::ProductionApprovalWorkflow => self.plan >= CommercialPlan::Pro,
            FeatureGate::TeamRoleBasedAccess => self.plan >= CommercialPlan::Team,
            FeatureGate::CentralizedMultiServerManagement => self.plan >= CommercialPlan::Team,
            FeatureGate::ExtendedAuditRetention => self.plan >= CommercialPlan::Team,
        }
    }

    /// Verifies access to a feature, returning a structured commercial entitlement error if blocked
    pub fn check_entitlement(&self, feature: &FeatureGate) -> Result<()> {
        if self.is_feature_allowed(feature) {
            Ok(())
        } else {
            let required_plan = match feature {
                FeatureGate::MultiWorkspace => "Developer ($9–15/mo)",
                FeatureGate::RemoteCloudControlPlane => "Developer ($9–15/mo)",
                FeatureGate::ProductionApprovalWorkflow => "Pro ($29–49/mo)",
                FeatureGate::TeamRoleBasedAccess => "Team ($79–149/mo)",
                FeatureGate::CentralizedMultiServerManagement => "Team ($79–149/mo)",
                FeatureGate::ExtendedAuditRetention => "Team ($79–149/mo)",
            };

            Err(SmooothError::PolicyViolation(format!(
                "Commercial entitlement required: Feature '{:?}' is not available on your current plan ({:?}). Upgrade to {} or higher.",
                feature, self.plan, required_plan
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_community_tier_limits() {
        let engine = EntitlementEngine::new(CommercialPlan::Community);
        assert!(!engine.is_feature_allowed(&FeatureGate::MultiWorkspace));
        assert!(!engine.is_feature_allowed(&FeatureGate::ProductionApprovalWorkflow));
        assert!(!engine.is_feature_allowed(&FeatureGate::TeamRoleBasedAccess));
        assert!(engine
            .check_entitlement(&FeatureGate::TeamRoleBasedAccess)
            .is_err());
    }

    #[test]
    fn test_developer_tier_entitlements() {
        let engine = EntitlementEngine::new(CommercialPlan::Developer);
        assert!(engine.is_feature_allowed(&FeatureGate::MultiWorkspace));
        assert!(engine.is_feature_allowed(&FeatureGate::RemoteCloudControlPlane));
        assert!(!engine.is_feature_allowed(&FeatureGate::ProductionApprovalWorkflow));
    }

    #[test]
    fn test_pro_and_team_tier_entitlements() {
        let pro = EntitlementEngine::new(CommercialPlan::Pro);
        assert!(pro.is_feature_allowed(&FeatureGate::ProductionApprovalWorkflow));
        assert!(!pro.is_feature_allowed(&FeatureGate::TeamRoleBasedAccess));

        let team = EntitlementEngine::new(CommercialPlan::Team);
        assert!(team.is_feature_allowed(&FeatureGate::TeamRoleBasedAccess));
        assert!(team.is_feature_allowed(&FeatureGate::CentralizedMultiServerManagement));
    }
}
