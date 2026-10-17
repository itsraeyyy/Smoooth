use serde::{Deserialize, Serialize};

fn default_expected_status() -> u16 {
    200
}

/// Structured health check specification for application workloads
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HealthSpec {
    /// Active HTTP GET probe against local container port and path
    Http {
        path: String,
        port: u16,
        #[serde(default = "default_expected_status")]
        expected_status: u16,
    },
    /// Active TCP socket connection probe against local port
    Tcp { port: u16 },
    /// Safe execution probe inside container using argument array (no shell parsing)
    Exec { cmd: Vec<String> },
    /// Basic process liveness probe (running container state)
    Liveness,
}
