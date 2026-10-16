use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AgentScope {
    ReadOnly,
    Plan,
    Deploy,
    Destructive,
    Admin,
}

impl AgentScope {
    pub fn parse_csv(csv: &str) -> Vec<AgentScope> {
        csv.split(',')
            .filter_map(|s| match s.trim().to_lowercase().as_str() {
                "read" | "read_only" | "readonly" => Some(AgentScope::ReadOnly),
                "plan" => Some(AgentScope::Plan),
                "deploy" => Some(AgentScope::Deploy),
                "destructive" | "destroy" => Some(AgentScope::Destructive),
                "admin" => Some(AgentScope::Admin),
                _ => None,
            })
            .collect()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AgentScope::ReadOnly => "read_only",
            AgentScope::Plan => "plan",
            AgentScope::Deploy => "deploy",
            AgentScope::Destructive => "destructive",
            AgentScope::Admin => "admin",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub agent_id: String,
    pub name: String,
    pub token_hash: String, // SHA-256 hash of token secret (token is NEVER stored in plaintext)
    pub token_prefix: String, // e.g. "smoooth_ag_a1b2..." first 12 characters for operator identification
    pub scopes: Vec<AgentScope>,
    pub workspace_id: String, // Bound workspace, or "*" for all workspaces
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

impl AgentIdentity {
    pub fn is_active(&self) -> bool {
        if self.revoked {
            return false;
        }
        if let Some(exp) = self.expires_at {
            if Utc::now() > exp {
                return false;
            }
        }
        true
    }

    pub fn has_scope(&self, required: &AgentScope) -> bool {
        if self.scopes.contains(&AgentScope::Admin) {
            return true;
        }
        self.scopes.contains(required)
    }

    pub fn allows_workspace(&self, target_ws: &str) -> bool {
        if self.workspace_id == "*" || self.workspace_id == target_ws {
            return true;
        }
        false
    }

    pub fn is_overprivileged(&self) -> bool {
        self.scopes.contains(&AgentScope::Admin) || self.scopes.contains(&AgentScope::Destructive)
    }
}
