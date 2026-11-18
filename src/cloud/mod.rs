pub mod entitlement;
pub mod rbac;

pub use entitlement::{CommercialPlan, EntitlementEngine, FeatureGate};
pub use rbac::TeamRole;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::engine::Orchestrator;
use crate::error::Result;
use crate::mcp::server::{CallerContext, McpServer};

/// Registration details connecting a self-hosted daemon to Smoooth Cloud
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonEnrollment {
    pub server_id: String,
    pub cloud_url: String,
    pub daemon_token: String,
    pub cluster_id: Option<String>,
    pub enrolled_at: DateTime<Utc>,
}

/// Periodic heartbeat status report sent from daemon to Smoooth Cloud control plane
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonHeartbeatReport {
    pub server_id: String,
    pub timestamp: DateTime<Utc>,
    pub is_healthy: bool,
    pub active_containers_count: usize,
    pub registered_resources_count: usize,
    pub uptime_seconds: u64,
}

/// Inbound operational command received from the cloud control plane
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCommand {
    pub command_id: String,
    pub action: String,
    pub payload: serde_json::Value,
    pub issued_at: DateTime<Utc>,
}

/// Outbound execution outcome sent back to the cloud control plane
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCommandOutcome {
    pub command_id: String,
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
    pub completed_at: DateTime<Utc>,
}

/// Client managing the outbound connection from customer VPS to Smoooth Cloud.
/// Implements ADR 0003: Outbound-only, zero open inbound ports on customer servers,
/// local-first execution authority, and strict local policy verification.
pub struct DaemonSyncClient {
    enrollment: DaemonEnrollment,
    orchestrator: Arc<Orchestrator>,
    mcp_server: Arc<McpServer>,
    start_time: std::time::Instant,
}

impl DaemonSyncClient {
    pub fn new(
        enrollment: DaemonEnrollment,
        orchestrator: Arc<Orchestrator>,
        mcp_server: Arc<McpServer>,
    ) -> Self {
        Self {
            enrollment,
            orchestrator,
            mcp_server,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn server_id(&self) -> &str {
        &self.enrollment.server_id
    }

    /// Generate a sanitized heartbeat telemetry payload for transmission to the cloud control plane
    pub async fn generate_heartbeat(&self) -> Result<DaemonHeartbeatReport> {
        let resources = self.orchestrator.list_resources("default").await;
        let active_count = resources
            .iter()
            .filter(|r| r.status == crate::models::ResourceStatus::Active)
            .count();

        Ok(DaemonHeartbeatReport {
            server_id: self.enrollment.server_id.clone(),
            timestamp: Utc::now(),
            is_healthy: true,
            active_containers_count: active_count,
            registered_resources_count: resources.len(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        })
    }

    /// Execute a command sent from the cloud control plane through the local MCP security boundary.
    /// Strict local policy rules (e.g. anti-self-approval and capability permits) are enforced locally on the VPS.
    pub async fn execute_cloud_command(&self, cmd: CloudCommand) -> CloudCommandOutcome {
        let caller = CallerContext {
            is_operator: true,
            identity: None,
        };

        match self
            .mcp_server
            .dispatch_tool_with_caller(&cmd.action, cmd.payload, &caller)
            .await
        {
            Ok(result) => CloudCommandOutcome {
                command_id: cmd.command_id,
                success: true,
                result,
                error: None,
                completed_at: Utc::now(),
            },
            Err(e) => CloudCommandOutcome {
                command_id: cmd.command_id,
                success: false,
                result: serde_json::Value::Null,
                error: Some(e.to_string()),
                completed_at: Utc::now(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::local::LocalStorage;

    #[tokio::test]
    async fn test_daemon_sync_client_local_execution_and_heartbeat() {
        let storage = Arc::new(LocalStorage::new_in_memory());
        let orchestrator = Arc::new(Orchestrator::new(storage));
        let mcp_server = Arc::new(McpServer::new(orchestrator.clone()));

        let enrollment = DaemonEnrollment {
            server_id: "srv_vps_us_east_1".to_string(),
            cloud_url: "https://cloud.smoooth.dev".to_string(),
            daemon_token: "smoooth_dm_test_token".to_string(),
            cluster_id: Some("cluster_alpha".to_string()),
            enrolled_at: Utc::now(),
        };

        let client = DaemonSyncClient::new(enrollment, orchestrator, mcp_server);
        assert_eq!(client.server_id(), "srv_vps_us_east_1");

        // 1. Heartbeat generation
        let heartbeat = client.generate_heartbeat().await.unwrap();
        assert!(heartbeat.is_healthy);
        assert_eq!(heartbeat.server_id, "srv_vps_us_east_1");

        // 2. Command execution through local security boundary
        let cmd = CloudCommand {
            command_id: "cmd_12345".to_string(),
            action: "list_capabilities".to_string(),
            payload: serde_json::json!({}),
            issued_at: Utc::now(),
        };

        let outcome = client.execute_cloud_command(cmd).await;
        assert!(outcome.success);
        assert_eq!(outcome.command_id, "cmd_12345");
        assert!(outcome.result.as_array().is_some());
    }
}
