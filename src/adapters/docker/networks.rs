use std::collections::HashMap;
use bollard::network::{CreateNetworkOptions, InspectNetworkOptions};

use crate::adapters::docker::client::DockerClient;
use crate::error::{Result, SmooothError};

pub struct NetworkManager;

impl NetworkManager {
    /// Ensure isolated bridge network exists for workspace/service communication
    pub async fn ensure_network(
        client: &DockerClient,
        network_name: &str,
        workspace_id: &str,
    ) -> Result<String> {
        let docker = client.inner();

        // 1. Check if network already exists
        if let Ok(net) = docker.inspect_network(network_name, None::<InspectNetworkOptions<String>>).await {
            if let Some(id) = net.id {
                return Ok(id);
            }
        }

        // 2. Create bridge network with labels
        let mut labels = HashMap::new();
        labels.insert("smoooth.managed".to_string(), "true".to_string());
        labels.insert("smoooth.workspace".to_string(), workspace_id.to_string());

        let options = CreateNetworkOptions {
            name: network_name.to_string(),
            driver: "bridge".to_string(),
            labels,
            check_duplicate: true,
            ..Default::default()
        };

        let response = docker
            .create_network(options)
            .await
            .map_err(|e| SmooothError::ContainerFailed(format!("Failed to create network '{}': {}", network_name, e)))?;

        Ok(response.id)
    }

    /// Remove network if no longer needed
    pub async fn remove_network(
        client: &DockerClient,
        network_name: &str,
    ) -> Result<()> {
        let docker = client.inner();
        let _ = docker.remove_network(network_name).await;
        Ok(())
    }
}
