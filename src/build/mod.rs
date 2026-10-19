//! Nixpacks Automated Build Engine for Smoooth
//! Enables zero-Dockerfile container image builds from raw source directories or Git repositories.
//! Supports automatic framework detection across Node.js/TypeScript, Python, PHP, Go, Rust, and more.

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use uuid::Uuid;

use crate::error::{Result, SmooothError};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NixpacksBuildOutput {
    pub success: bool,
    pub image_tag: String,
    pub detected_provider: Option<String>,
    pub logs: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NixpacksPlanOutput {
    pub providers: Vec<String>,
    pub variables: HashMap<String, String>,
    pub phases: serde_json::Value,
    pub start: Option<serde_json::Value>,
}

pub struct NixpacksBuilder;

impl NixpacksBuilder {
    /// Check whether nixpacks is available on the host PATH
    pub async fn is_available() -> bool {
        Command::new("which")
            .arg("nixpacks")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Generate a Nixpacks build plan without building
    pub async fn plan(source: &str) -> Result<serde_json::Value> {
        let is_git = source.starts_with("http://")
            || source.starts_with("https://")
            || source.starts_with("git@");

        let (working_dir, cleanup_dir) = if is_git {
            let tmp_id = Uuid::new_v4().simple().to_string();
            let tmp_path = format!("/tmp/smoooth-nixplan-{}", tmp_id);

            let git_clone = Command::new("git")
                .args(["clone", "--depth", "1", source, &tmp_path])
                .output()
                .await
                .map_err(|e| SmooothError::ExecutionFailed(format!("Failed to clone Git repository '{}': {}", source, e)))?;

            if !git_clone.status.success() {
                let err = String::from_utf8_lossy(&git_clone.stderr);
                return Err(SmooothError::ExecutionFailed(format!("Git clone failed for '{}': {}", source, err)));
            }

            (tmp_path.clone(), Some(tmp_path))
        } else {
            if !Path::new(source).exists() {
                return Err(SmooothError::Validation(format!(
                    "Source path '{}' does not exist",
                    source
                )));
            }
            (source.to_string(), None)
        };

        let output = if Self::is_available().await {
            Command::new("nixpacks")
                .arg("plan")
                .arg(&working_dir)
                .output()
                .await
                .map_err(|e| SmooothError::ExecutionFailed(format!("Failed to execute nixpacks plan: {}", e)))
        } else {
            // Fallback to containerized nixpacks
            Command::new("docker")
                .args([
                    "run", "--rm",
                    "-v", &format!("{}:/app:ro", working_dir),
                    "ghcr.io/railwayapp/nixpacks:latest",
                    "plan", "/app",
                ])
                .output()
                .await
                .map_err(|e| SmooothError::ExecutionFailed(format!("Failed to run docker nixpacks plan: {}", e)))
        };

        // Clean up temporary clone if present
        if let Some(dir) = cleanup_dir {
            let _ = tokio::fs::remove_dir_all(&dir).await;
        }

        let output = output?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SmooothError::ExecutionFailed(format!("nixpacks plan failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str::<serde_json::Value>(&stdout)
            .map_err(|e| SmooothError::Validation(format!("Failed to parse nixpacks plan JSON: {}. Output was: {}", e, stdout)))
    }

    /// Build a Docker image from a local path or Git repository
    pub async fn build(
        source: &str,
        image_tag: &str,
        env_vars: Option<HashMap<String, String>>,
    ) -> Result<NixpacksBuildOutput> {
        let is_git = source.starts_with("http://")
            || source.starts_with("https://")
            || source.starts_with("git@");

        let (working_dir, cleanup_dir) = if is_git {
            let tmp_id = Uuid::new_v4().simple().to_string();
            let tmp_path = format!("/tmp/smoooth-nixpacks-{}", tmp_id);

            let git_clone = Command::new("git")
                .args(["clone", "--depth", "1", source, &tmp_path])
                .output()
                .await
                .map_err(|e| SmooothError::ExecutionFailed(format!("Failed to clone Git repository '{}': {}", source, e)))?;

            if !git_clone.status.success() {
                let err = String::from_utf8_lossy(&git_clone.stderr);
                return Err(SmooothError::ExecutionFailed(format!("Git clone failed for '{}': {}", source, err)));
            }

            (tmp_path.clone(), Some(tmp_path))
        } else {
            if !Path::new(source).exists() {
                return Err(SmooothError::Validation(format!(
                    "Source directory '{}' does not exist",
                    source
                )));
            }
            (source.to_string(), None)
        };

        let mut logs = Vec::new();
        logs.push(format!("[smoooth-build] Source target: {}", source));
        logs.push(format!("[smoooth-build] Output image: {}", image_tag));

        // Detect provider / plan first
        let detected_provider = match Self::plan(&working_dir).await {
            Ok(p) => {
                let prov = p.get("variables")
                    .and_then(|v| v.get("NIXPACKS_METADATA"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string());
                logs.push(format!("[smoooth-build] Detected stack: {}", prov.as_deref().unwrap_or("generic")));
                prov
            }
            Err(_) => None,
        };

        // Construct build command
        let use_native = Self::is_available().await;
        let mut cmd = if use_native {
            let mut c = Command::new("nixpacks");
            c.arg("build").arg(&working_dir).arg("--name").arg(image_tag);
            if let Some(envs) = &env_vars {
                for (k, v) in envs {
                    c.arg("--env").arg(format!("{}={}", k, v));
                }
            }
            c
        } else {
            let mut c = Command::new("docker");
            c.args([
                "run", "--rm",
                "-v", "/var/run/docker.sock:/var/run/docker.sock",
                "-v", &format!("{}:/app:ro", working_dir),
                "ghcr.io/railwayapp/nixpacks:latest",
                "build", "/app", "--name", image_tag,
            ]);
            if let Some(envs) = &env_vars {
                for (k, v) in envs {
                    c.args(["--env", &format!("{}={}", k, v)]);
                }
            }
            c
        };

        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            SmooothError::ExecutionFailed(format!("Failed to spawn nixpacks build process: {}", e))
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            SmooothError::ExecutionFailed("Failed to capture nixpacks build stdout".to_string())
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            SmooothError::ExecutionFailed("Failed to capture nixpacks build stderr".to_string())
        })?;

        let mut stdout_reader = BufReader::new(stdout).lines();
        let mut stderr_reader = BufReader::new(stderr).lines();

        loop {
            tokio::select! {
                line = stdout_reader.next_line() => {
                    match line {
                        Ok(Some(l)) => logs.push(l),
                        Ok(None) => break,
                        Err(e) => {
                            logs.push(format!("[error] {}", e));
                            break;
                        }
                    }
                }
                line = stderr_reader.next_line() => {
                    match line {
                        Ok(Some(l)) => logs.push(format!("[stderr] {}", l)),
                        Ok(None) => {},
                        Err(e) => logs.push(format!("[stderr error] {}", e)),
                    }
                }
            }
        }

        let status = child.wait().await.map_err(|e| {
            SmooothError::ExecutionFailed(format!("Failed waiting for nixpacks build: {}", e))
        })?;

        // Cleanup temporary git clone if needed
        if let Some(dir) = cleanup_dir {
            let _ = tokio::fs::remove_dir_all(&dir).await;
        }

        if status.success() {
            logs.push(format!("[smoooth-build] Image '{}' successfully built and tagged.", image_tag));
            Ok(NixpacksBuildOutput {
                success: true,
                image_tag: image_tag.to_string(),
                detected_provider,
                logs,
                error: None,
            })
        } else {
            let err_msg = format!("Nixpacks build failed with exit code: {:?}", status.code());
            logs.push(format!("[smoooth-build] {}", err_msg));
            Ok(NixpacksBuildOutput {
                success: false,
                image_tag: image_tag.to_string(),
                detected_provider,
                logs,
                error: Some(err_msg),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_nixpacks_plan_on_nonexistent_path_fails() {
        let res = NixpacksBuilder::plan("/tmp/nonexistent_test_path_12345").await;
        assert!(res.is_err());
    }
}
