use bollard::Docker;
use crate::error::{Result, SmooothError};

#[derive(Clone)]
pub struct DockerClient {
    client: Docker,
}

impl DockerClient {
    /// Try multiple discovery & connection strategies for Windows, Linux, and custom endpoints
    pub fn try_new() -> Result<Self> {
        #[cfg(windows)]
        let client = Docker::connect_with_defaults()
            .or_else(|_| Docker::connect_with_named_pipe("//./pipe/dockerDesktopLinuxEngine", 120, bollard::API_DEFAULT_VERSION))
            .or_else(|_| Docker::connect_with_named_pipe("//./pipe/docker_engine", 120, bollard::API_DEFAULT_VERSION))
            .or_else(|_| Docker::connect_with_http("http://127.0.0.1:2375", 120, bollard::API_DEFAULT_VERSION));

        #[cfg(unix)]
        let client = Docker::connect_with_socket_defaults()
            .or_else(|_| Docker::connect_with_defaults())
            .or_else(|_| Docker::connect_with_http("http://127.0.0.1:2375", 120, bollard::API_DEFAULT_VERSION));

        #[cfg(not(any(windows, unix)))]
        let client = Docker::connect_with_defaults();

        let client = client.map_err(|e| SmooothError::DockerUnavailable(format!("Could not connect to Docker daemon: {}", e)))?;

        Ok(Self { client })
    }

    pub fn inner(&self) -> &Docker {
        &self.client
    }

    pub async fn ping(&self) -> Result<()> {
        self.client
            .ping()
            .await
            .map_err(|e| SmooothError::DockerUnavailable(format!("Docker daemon ping failed: {}", e)))?;
        Ok(())
    }

    pub async fn version_info(&self) -> Result<String> {
        let v = self.client.version().await
            .map_err(|e| SmooothError::DockerUnavailable(format!("Docker version query failed: {}", e)))?;
        Ok(v.version.unwrap_or_else(|| "unknown".to_string()))
    }
}
