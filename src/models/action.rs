use serde::{Deserialize, Serialize};

/// High-precision, parameterized actions in the Smoooth capability authority system.
///
/// Invariants:
/// - Every action has strict, mathematically verifiable attenuation rules.
/// - A delegated child action can only maintain or narrow permissions, never broaden them.
/// - Limits can only decrease, key sets can only shrink, patterns can only specialize,
///   and destruction scopes can only narrow from persistent to resource-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", content = "parameters", rename_all = "snake_case")]
pub enum Action {
    /// Read non-sensitive metadata, status, and health metrics
    InspectMetadata,

    /// Read stdout/stderr logs from a specific container workload
    ReadLogs { tail_limit: Option<u32> },

    /// Read a specific secret value (requires exact key designation)
    ReadSecret { key_name: String, purpose: String },

    /// Update workload environment variables without exposing existing secrets
    UpdateEnv { allowed_keys: Vec<String> },

    /// Restart a running container workload
    RestartWorkload,

    /// Provision a workload or service with mandatory bounds
    Provision {
        resource_type: ResourceType,
        name_pattern: String,
        limits: ResourceLimits,
        allow_public_exposure: bool,
    },

    /// Trigger a database or volume snapshot
    Snapshot { allow_external_export: bool },

    /// Reconcile state drift back to approved specification
    Reconcile,

    /// Destroy infrastructure with explicit persistent data guard
    Destroy(DestructionScope),
}

/// Target infrastructure resource classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceType {
    Workload,
    Database,
    Network,
    Volume,
    Secret,
    Ingress,
}

/// Guarded scope for infrastructure destruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DestructionScope {
    /// Destroy container/process only; MUST preserve database volumes and snapshots
    ResourceOnly,
    /// Permanently destroy persistent volumes and data (requires operator approval)
    ResourceAndPersistentData,
}

/// Bounded computational constraints for provisioning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: i64,
    pub max_cpu_millicores: u32,
    pub max_pids: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 536_870_912, // 512 MB
            max_cpu_millicores: 1000,      // 1.0 CPU
            max_pids: 256,
        }
    }
}

impl Action {
    /// Evaluates monotonic attenuation from `self` (parent grant) to `child`.
    ///
    /// Returns `true` if and only if `child` is equal to or strictly narrower than `self`.
    pub fn can_attenuate_to(&self, child: &Action) -> bool {
        match (self, child) {
            (Action::InspectMetadata, Action::InspectMetadata) => true,

            (
                Action::ReadLogs {
                    tail_limit: parent_tail,
                },
                Action::ReadLogs {
                    tail_limit: child_tail,
                },
            ) => match (parent_tail, child_tail) {
                (None, _) => true,        // Parent is unlimited; child can be unlimited or bounded
                (Some(_), None) => false, // Child cannot escalate bounded tail to unlimited!
                (Some(p), Some(c)) => *c <= *p, // Child limit cannot exceed parent limit
            },

            (
                Action::ReadSecret {
                    key_name: p_key,
                    purpose: p_purpose,
                },
                Action::ReadSecret {
                    key_name: c_key,
                    purpose: c_purpose,
                },
            ) => p_key == c_key && p_purpose == c_purpose,

            (
                Action::UpdateEnv {
                    allowed_keys: p_keys,
                },
                Action::UpdateEnv {
                    allowed_keys: c_keys,
                },
            ) => {
                // Every allowed key in child must be present in parent
                c_keys.iter().all(|k| p_keys.contains(k))
            }

            (Action::RestartWorkload, Action::RestartWorkload) => true,

            (
                Action::Provision {
                    resource_type: p_type,
                    name_pattern: p_pat,
                    limits: p_limits,
                    allow_public_exposure: p_exp,
                },
                Action::Provision {
                    resource_type: c_type,
                    name_pattern: c_pat,
                    limits: c_limits,
                    allow_public_exposure: c_exp,
                },
            ) => {
                if p_type != c_type {
                    return false;
                }
                // If parent disallows public exposure, child cannot enable it
                if !p_exp && *c_exp {
                    return false;
                }
                // Child limits cannot exceed parent limits
                if c_limits.max_memory_bytes > p_limits.max_memory_bytes
                    || c_limits.max_cpu_millicores > p_limits.max_cpu_millicores
                    || c_limits.max_pids > p_limits.max_pids
                {
                    return false;
                }
                // Name pattern must be equal or narrower
                is_subpattern(p_pat, c_pat)
            }

            (
                Action::Snapshot {
                    allow_external_export: p_exp,
                },
                Action::Snapshot {
                    allow_external_export: c_exp,
                },
            ) => {
                // If parent disallows export, child cannot enable it
                if !p_exp && *c_exp {
                    return false;
                }
                true
            }

            (Action::Reconcile, Action::Reconcile) => true,

            (Action::Destroy(p_scope), Action::Destroy(c_scope)) => match (p_scope, c_scope) {
                (DestructionScope::ResourceAndPersistentData, _) => true,
                (DestructionScope::ResourceOnly, DestructionScope::ResourceOnly) => true,
                (DestructionScope::ResourceOnly, DestructionScope::ResourceAndPersistentData) => {
                    false
                }
            },

            _ => false, // Mismatched action variants cannot attenuate
        }
    }
}

/// Helper evaluating whether `child_pat` is equal to or narrower than `parent_pat`.
pub fn is_subpattern(parent_pat: &str, child_pat: &str) -> bool {
    if parent_pat == "*" {
        // Universal wildcard parent encompasses any valid pattern
        return true;
    }
    if parent_pat == child_pat {
        return true;
    }
    if let Some(prefix) = parent_pat.strip_suffix('*') {
        // Parent is a prefix wildcard, e.g. "web-*"
        // Child must start with that prefix
        child_pat.starts_with(prefix)
    } else {
        // Exact pattern requires exact match (child cannot broaden to wildcard)
        false
    }
}

/// Evaluates monotonic attenuation for a set of actions.
///
/// Returns `true` if and only if every action in `child_actions` is authorized by
/// at least one action in `parent_actions` through valid monotonic attenuation.
pub fn can_attenuate_action_set(parent_actions: &[Action], child_actions: &[Action]) -> bool {
    child_actions.iter().all(|child| {
        parent_actions
            .iter()
            .any(|parent| parent.can_attenuate_to(child))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspect_metadata_attenuation() {
        assert!(Action::InspectMetadata.can_attenuate_to(&Action::InspectMetadata));
        assert!(!Action::InspectMetadata.can_attenuate_to(&Action::RestartWorkload));
    }

    #[test]
    fn test_read_logs_attenuation_narrowing() {
        let unlimited = Action::ReadLogs { tail_limit: None };
        let bound_100 = Action::ReadLogs {
            tail_limit: Some(100),
        };
        let bound_50 = Action::ReadLogs {
            tail_limit: Some(50),
        };
        let bound_200 = Action::ReadLogs {
            tail_limit: Some(200),
        };

        // Unlimited can attenuate to unlimited or bounded
        assert!(unlimited.can_attenuate_to(&unlimited));
        assert!(unlimited.can_attenuate_to(&bound_100));

        // Bounded can attenuate to smaller or equal bound
        assert!(bound_100.can_attenuate_to(&bound_100));
        assert!(bound_100.can_attenuate_to(&bound_50));

        // Bounded CANNOT escalate to larger bound or unlimited
        assert!(!bound_100.can_attenuate_to(&bound_200));
        assert!(!bound_100.can_attenuate_to(&unlimited));
    }

    #[test]
    fn test_read_secret_attenuation() {
        let parent = Action::ReadSecret {
            key_name: "DATABASE_URL".into(),
            purpose: "replication_check".into(),
        };
        let same_child = Action::ReadSecret {
            key_name: "DATABASE_URL".into(),
            purpose: "replication_check".into(),
        };
        let different_key = Action::ReadSecret {
            key_name: "API_KEY".into(),
            purpose: "replication_check".into(),
        };
        let different_purpose = Action::ReadSecret {
            key_name: "DATABASE_URL".into(),
            purpose: "arbitrary_dump".into(),
        };

        assert!(parent.can_attenuate_to(&same_child));
        assert!(!parent.can_attenuate_to(&different_key));
        assert!(!parent.can_attenuate_to(&different_purpose));
    }

    #[test]
    fn test_update_env_attenuation_subset_keys() {
        let parent = Action::UpdateEnv {
            allowed_keys: vec!["PORT".into(), "LOG_LEVEL".into(), "NODE_ENV".into()],
        };
        let subset_child = Action::UpdateEnv {
            allowed_keys: vec!["PORT".into(), "LOG_LEVEL".into()],
        };
        let single_child = Action::UpdateEnv {
            allowed_keys: vec!["LOG_LEVEL".into()],
        };
        let empty_child = Action::UpdateEnv {
            allowed_keys: vec![],
        };
        let expanded_child = Action::UpdateEnv {
            allowed_keys: vec!["PORT".into(), "SECRET_KEY".into()],
        };

        assert!(parent.can_attenuate_to(&subset_child));
        assert!(parent.can_attenuate_to(&single_child));
        assert!(parent.can_attenuate_to(&empty_child));

        // Adding a new key not present in parent must fail
        assert!(!parent.can_attenuate_to(&expanded_child));
    }

    #[test]
    fn test_provision_attenuation_limits_and_exposure() {
        let parent_limits = ResourceLimits {
            max_memory_bytes: 1_073_741_824, // 1 GB
            max_cpu_millicores: 2000,        // 2.0 CPU
            max_pids: 512,
        };
        let parent = Action::Provision {
            resource_type: ResourceType::Workload,
            name_pattern: "web-*".into(),
            limits: parent_limits,
            allow_public_exposure: false,
        };

        // 1. Equal child allowed
        assert!(parent.can_attenuate_to(&parent));

        // 2. Narrower limits allowed
        let narrower_limits = ResourceLimits {
            max_memory_bytes: 536_870_912, // 512 MB
            max_cpu_millicores: 1000,      // 1.0 CPU
            max_pids: 256,
        };
        let child_narrower = Action::Provision {
            resource_type: ResourceType::Workload,
            name_pattern: "web-api".into(),
            limits: narrower_limits,
            allow_public_exposure: false,
        };
        assert!(parent.can_attenuate_to(&child_narrower));

        // 3. Escalated memory rejected
        let mut escalated_limits = parent_limits;
        escalated_limits.max_memory_bytes = 2_147_483_648; // 2 GB
        let child_escalated_mem = Action::Provision {
            resource_type: ResourceType::Workload,
            name_pattern: "web-*".into(),
            limits: escalated_limits,
            allow_public_exposure: false,
        };
        assert!(!parent.can_attenuate_to(&child_escalated_mem));

        // 4. Escalated public exposure rejected
        let child_escalated_exposure = Action::Provision {
            resource_type: ResourceType::Workload,
            name_pattern: "web-*".into(),
            limits: parent_limits,
            allow_public_exposure: true,
        };
        assert!(!parent.can_attenuate_to(&child_escalated_exposure));

        // 5. Mismatched resource type rejected
        let child_db = Action::Provision {
            resource_type: ResourceType::Database,
            name_pattern: "web-*".into(),
            limits: parent_limits,
            allow_public_exposure: false,
        };
        assert!(!parent.can_attenuate_to(&child_db));

        // 6. Broadened pattern rejected
        let child_broad_pattern = Action::Provision {
            resource_type: ResourceType::Workload,
            name_pattern: "*".into(),
            limits: parent_limits,
            allow_public_exposure: false,
        };
        assert!(!parent.can_attenuate_to(&child_broad_pattern));
    }

    #[test]
    fn test_destroy_scope_attenuation() {
        let parent_persistent = Action::Destroy(DestructionScope::ResourceAndPersistentData);
        let child_resource_only = Action::Destroy(DestructionScope::ResourceOnly);

        // Persistent destruction can attenuate to ResourceOnly or remain Persistent
        assert!(parent_persistent.can_attenuate_to(&parent_persistent));
        assert!(parent_persistent.can_attenuate_to(&child_resource_only));

        // ResourceOnly CANNOT escalate to persistent data deletion!
        assert!(child_resource_only.can_attenuate_to(&child_resource_only));
        assert!(!child_resource_only.can_attenuate_to(&parent_persistent));
    }

    #[test]
    fn test_snapshot_export_attenuation() {
        let parent_no_export = Action::Snapshot {
            allow_external_export: false,
        };
        let parent_with_export = Action::Snapshot {
            allow_external_export: true,
        };

        assert!(parent_with_export.can_attenuate_to(&parent_with_export));
        assert!(parent_with_export.can_attenuate_to(&parent_no_export));

        assert!(parent_no_export.can_attenuate_to(&parent_no_export));
        // Disallowed export cannot be escalated to allowed export
        assert!(!parent_no_export.can_attenuate_to(&parent_with_export));
    }

    #[test]
    fn test_action_set_attenuation() {
        let parent_set = vec![
            Action::InspectMetadata,
            Action::ReadLogs {
                tail_limit: Some(100),
            },
            Action::Destroy(DestructionScope::ResourceOnly),
        ];

        let valid_child_set = vec![
            Action::InspectMetadata,
            Action::ReadLogs {
                tail_limit: Some(50),
            },
        ];
        assert!(can_attenuate_action_set(&parent_set, &valid_child_set));

        let empty_child_set = vec![];
        assert!(can_attenuate_action_set(&parent_set, &empty_child_set));

        // Child requesting unpermitted action fails
        let invalid_child_new_action = vec![Action::InspectMetadata, Action::RestartWorkload];
        assert!(!can_attenuate_action_set(
            &parent_set,
            &invalid_child_new_action
        ));

        // Child requesting escalated action fails
        let invalid_child_escalated =
            vec![Action::Destroy(DestructionScope::ResourceAndPersistentData)];
        assert!(!can_attenuate_action_set(
            &parent_set,
            &invalid_child_escalated
        ));
    }
}
