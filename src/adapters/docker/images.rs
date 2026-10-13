use std::time::Duration;
use bollard::image::CreateImageOptions;
use futures_util::StreamExt;
use tokio::time::timeout;

use crate::adapters::docker::client::DockerClient;
use crate::error::{Result, SmooothError};

pub struct ImageManager;

impl ImageManager {
    /// Pull image with optional immutable digest, bounded by a 300s timeout
    pub async fn ensure_image(
        client: &DockerClient,
        image_name: &str,
        pinned_digest: Option<&str>,
    ) -> Result<()> {
        let full_reference = match pinned_digest {
            Some(digest) => format!("{}@{}", image_name, digest),
            None => image_name.to_string(),
        };

        let options = CreateImageOptions {
            from_image: full_reference.clone(),
            ..Default::default()
        };

        let pull_future = async {
            let mut stream = client.inner().create_image(Some(options), None, None);
            while let Some(result) = stream.next().await {
                if let Err(e) = result {
                    return Err(SmooothError::ContainerFailed(format!(
                        "Failed while pulling image '{}': {}",
                        full_reference, e
                    )));
                }
            }
            Ok(())
        };

        // Bounded 5-minute timeout for image pulls
        timeout(Duration::from_secs(300), pull_future)
            .await
            .map_err(|_| {
                SmooothError::Timeout(format!("Image pull for '{}' timed out after 300s", full_reference))
            })?
    }
}
