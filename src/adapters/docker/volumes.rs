use std::collections::HashMap;
use bollard::volume::{CreateVolumeOptions, RemoveVolumeOptions};

use crate::adapters::docker::client::DockerClient;
use crate::error::{Result, SmooothError};

pub struct VolumeManager;

impl VolumeManager {
    /// Ensure named volume exists with persistent lifecycle metadata labels
    pub async fn ensure_volume(
        client: &DockerClient,
        volume_name: &str,
        workspace_id: &str,
        is_persistent: bool,
    ) -> Result<String> {
        if volume_name.starts_with('/')
            || volume_name.starts_with('\\')
            || volume_name.contains('/')
            || volume_name.contains('\\')
            || volume_name.starts_with('.')
        {
            return Err(SmooothError::PolicyViolation(format!(
                "Host path bind mounts are strictly forbidden ('{}'). Only named Docker volumes are permitted.",
                volume_name
            )));
        }

        let docker = client.inner();

        // 1. Check if volume already exists
        if let Ok(vol) = docker.inspect_volume(volume_name).await {
            return Ok(vol.name);
        }

        // 2. Create volume with explicit ownership & persistence labels
        let mut labels = HashMap::new();
        labels.insert("smoooth.managed".to_string(), "true".to_string());
        labels.insert("smoooth.workspace".to_string(), workspace_id.to_string());
        labels.insert(
            "smoooth.persistent".to_string(),
            if is_persistent { "true".to_string() } else { "false".to_string() },
        );

        let options = CreateVolumeOptions {
            name: volume_name.to_string(),
            labels,
            ..Default::default()
        };

        let created = docker
            .create_volume(options)
            .await
            .map_err(|e| SmooothError::ContainerFailed(format!("Failed to create volume '{}': {}", volume_name, e)))?;

        Ok(created.name)
    }

    /// Safely remove volume, guarding against deletion if tagged persistent unless explicitly forced
    pub async fn remove_volume(
        client: &DockerClient,
        volume_name: &str,
        force_destroy_persistent: bool,
    ) -> Result<()> {
        let docker = client.inner();

        if let Ok(vol) = docker.inspect_volume(volume_name).await {
            let is_persistent = vol.labels.get("smoooth.persistent").map(|v| v == "true").unwrap_or(false);
            if is_persistent && !force_destroy_persistent {
                return Err(SmooothError::PolicyViolation(format!(
                    "Volume '{}' is tagged persistent. Routine deletion blocked.",
                    volume_name
                )));
            }
        }

        docker
            .remove_volume(volume_name, None::<RemoveVolumeOptions>)
            .await
            .map_err(|e| SmooothError::ContainerFailed(format!("Failed to remove volume '{}': {}", volume_name, e)))?;

        Ok(())
    }
}
