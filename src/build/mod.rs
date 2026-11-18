//! Nixpacks Automated Build Engine for Smoooth
//! Enables zero-Dockerfile container image builds from raw source directories or Git repositories.
//! Supports automatic framework detection across Node.js/TypeScript, Python, PHP, Go, Rust, and more.
//!
//! Security standard:
//! - Build containers NEVER receive access to the host Docker socket (`/var/run/docker.sock`).
//! - Git repository clone URLs are strictly validated and separated with `--` to prevent argument injection.
//! - Image tags are strictly validated to prevent command injection and option injection.

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

/// Validates target Docker image tag for syntax and prevents command/option injection
pub fn validate_image_tag(tag: &str) -> Result<()> {
    let t = tag.trim();
    if t.is_empty() || t.len() > 128 {
        return Err(SmooothError::Validation(
            "Image tag must be between 1 and 128 characters".to_string(),
        ));
    }
    if t.starts_with('-') || t.starts_with('.') {
        return Err(SmooothError::Validation(
            "Image tag cannot start with a dash or dot".to_string(),
        ));
    }
    let valid = t.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == ':' || c == '/' || c == '.' || c == '-' || c == '_'
    });
    if !valid {
        return Err(SmooothError::Validation(format!(
            "Image tag '{}' contains invalid characters. Only alphanumeric, ':', '/', '.', '-', '_' are permitted.",
            tag
        )));
    }
    Ok(())
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
            if source.starts_with('-') {
                return Err(SmooothError::Validation(
                    "Git repository source URL cannot start with a dash".to_string(),
                ));
            }
            let tmp_id = Uuid::new_v4().simple().to_string();
            let tmp_path = format!("/tmp/smoooth-nixplan-{}", tmp_id);

            let git_clone = Command::new("git")
                .args(["clone", "--depth", "1", "--", source, &tmp_path])
                .output()
                .await
                .map_err(|e| {
                    SmooothError::ExecutionFailed(format!(
                        "Failed to clone Git repository '{}': {}",
                        source, e
                    ))
                })?;

            if !git_clone.status.success() {
                let err = String::from_utf8_lossy(&git_clone.stderr);
                return Err(SmooothError::ExecutionFailed(format!(
                    "Git clone failed for '{}': {}",
                    source, err
                )));
            }

            (tmp_path.clone(), Some(tmp_path))
        } else {
            let path = Path::new(source);
            if !path.exists() {
                return Err(SmooothError::Validation(format!(
                    "Source path '{}' does not exist",
                    source
                )));
            }
            if !path.is_dir() {
                return Err(SmooothError::Validation(format!(
                    "Source path '{}' is not a directory",
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
                .map_err(|e| {
                    SmooothError::ExecutionFailed(format!("Failed to execute nixpacks plan: {}", e))
                })
        } else {
            // Containerized nixpacks: Read-only mount, no docker socket, no-new-privileges
            Command::new("docker")
                .args([
                    "run",
                    "--rm",
                    "--security-opt",
                    "no-new-privileges:true",
                    "-v",
                    &format!("{}:/app:ro", working_dir),
                    "ghcr.io/railwayapp/nixpacks:latest",
                    "plan",
                    "/app",
                ])
                .output()
                .await
                .map_err(|e| {
                    SmooothError::ExecutionFailed(format!(
                        "Failed to run docker nixpacks plan: {}",
                        e
                    ))
                })
        };

        // Clean up temporary clone if present
        if let Some(dir) = cleanup_dir {
            let _ = tokio::fs::remove_dir_all(&dir).await;
        }

        let output = output?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SmooothError::ExecutionFailed(format!(
                "nixpacks plan failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str::<serde_json::Value>(&stdout).map_err(|e| {
            SmooothError::Validation(format!(
                "Failed to parse nixpacks plan JSON: {}. Output was: {}",
                e, stdout
            ))
        })
    }

    /// Build a Docker image from a local path or Git repository with zero socket exposure
    pub async fn build(
        source: &str,
        image_tag: &str,
        env_vars: Option<HashMap<String, String>>,
    ) -> Result<NixpacksBuildOutput> {
        // Enforce strict image tag validation
        validate_image_tag(image_tag)?;

        let is_git = source.starts_with("http://")
            || source.starts_with("https://")
            || source.starts_with("git@");

        let (working_dir, cleanup_dir) = if is_git {
            if source.starts_with('-') {
                return Err(SmooothError::Validation(
                    "Git repository source URL cannot start with a dash".to_string(),
                ));
            }
            let tmp_id = Uuid::new_v4().simple().to_string();
            let tmp_path = format!("/tmp/smoooth-nixpacks-{}", tmp_id);

            let git_clone = Command::new("git")
                .args(["clone", "--depth", "1", "--", source, &tmp_path])
                .output()
                .await
                .map_err(|e| {
                    SmooothError::ExecutionFailed(format!(
                        "Failed to clone Git repository '{}': {}",
                        source, e
                    ))
                })?;

            if !git_clone.status.success() {
                let err = String::from_utf8_lossy(&git_clone.stderr);
                return Err(SmooothError::ExecutionFailed(format!(
                    "Git clone failed for '{}': {}",
                    source, err
                )));
            }

            (tmp_path.clone(), Some(tmp_path))
        } else {
            let path = Path::new(source);
            if !path.exists() {
                return Err(SmooothError::Validation(format!(
                    "Source directory '{}' does not exist",
                    source
                )));
            }
            if !path.is_dir() {
                return Err(SmooothError::Validation(format!(
                    "Source directory '{}' is not a directory",
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
                let prov = p
                    .get("variables")
                    .and_then(|v| v.get("NIXPACKS_METADATA"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string());
                logs.push(format!(
                    "[smoooth-build] Detected stack: {}",
                    prov.as_deref().unwrap_or("generic")
                ));
                prov
            }
            Err(_) => None,
        };

        let use_native = Self::is_available().await;

        let status = if use_native {
            let mut c = Command::new("nixpacks");
            c.arg("build")
                .arg(&working_dir)
                .arg("--name")
                .arg(image_tag);
            if let Some(envs) = &env_vars {
                for (k, v) in envs {
                    c.arg("--env").arg(format!("{}={}", k, v));
                }
            }
            c.stdout(Stdio::piped()).stderr(Stdio::piped());

            let mut child = c.spawn().map_err(|e| {
                SmooothError::ExecutionFailed(format!(
                    "Failed to spawn native nixpacks build process: {}",
                    e
                ))
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

            child.wait().await.map_err(|e| {
                SmooothError::ExecutionFailed(format!("Failed waiting for nixpacks build: {}", e))
            })?
        } else {
            // Containerized build with ZERO Docker socket exposure:
            // Step 1: Generate Dockerfile into isolated temporary output directory
            let out_id = Uuid::new_v4().simple().to_string();
            let out_path = format!("/tmp/smoooth-nixout-{}", out_id);
            let _ = tokio::fs::create_dir_all(&out_path).await;

            logs.push("[smoooth-build] Generating container artifacts in isolated sandbox (no socket mount)...".to_string());
            let mut gen_cmd = Command::new("docker");
            gen_cmd.args([
                "run",
                "--rm",
                "--security-opt",
                "no-new-privileges:true",
                "-v",
                &format!("{}:/app:ro", working_dir),
                "-v",
                &format!("{}:/out:rw", out_path),
                "ghcr.io/railwayapp/nixpacks:latest",
                "build",
                "/app",
                "--out",
                "/out",
            ]);
            if let Some(envs) = &env_vars {
                for (k, v) in envs {
                    gen_cmd.args(["--env", &format!("{}={}", k, v)]);
                }
            }

            let gen_output = gen_cmd.output().await.map_err(|e| {
                SmooothError::ExecutionFailed(format!(
                    "Failed executing isolated nixpacks generator: {}",
                    e
                ))
            })?;

            if !gen_output.status.success() {
                let stderr = String::from_utf8_lossy(&gen_output.stderr);
                logs.push(format!(
                    "[smoooth-build error] Nixpacks artifact generation failed: {}",
                    stderr
                ));
                let _ = tokio::fs::remove_dir_all(&out_path).await;
                if let Some(dir) = cleanup_dir {
                    let _ = tokio::fs::remove_dir_all(&dir).await;
                }
                return Ok(NixpacksBuildOutput {
                    success: false,
                    image_tag: image_tag.to_string(),
                    detected_provider,
                    logs,
                    error: Some(format!("Nixpacks generation failed: {}", stderr)),
                });
            }

            // Step 2: Build image on host Docker from generated artifacts (standard docker build)
            logs.push(format!(
                "[smoooth-build] Building image '{}' from generated Dockerfile...",
                image_tag
            ));
            let mut build_cmd = Command::new("docker");
            build_cmd.args(["build", "-t", image_tag, &out_path]);
            build_cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

            let mut build_child = build_cmd.spawn().map_err(|e| {
                SmooothError::ExecutionFailed(format!("Failed spawning docker build: {}", e))
            })?;

            let stdout = build_child.stdout.take().ok_or_else(|| {
                SmooothError::ExecutionFailed("Failed capturing docker build stdout".to_string())
            })?;
            let stderr = build_child.stderr.take().ok_or_else(|| {
                SmooothError::ExecutionFailed("Failed capturing docker build stderr".to_string())
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

            let b_status = build_child.wait().await.map_err(|e| {
                SmooothError::ExecutionFailed(format!("Failed waiting for docker build: {}", e))
            })?;

            let _ = tokio::fs::remove_dir_all(&out_path).await;
            b_status
        };

        // Cleanup temporary git clone if needed
        if let Some(dir) = cleanup_dir {
            let _ = tokio::fs::remove_dir_all(&dir).await;
        }

        if status.success() {
            logs.push(format!(
                "[smoooth-build] Image '{}' successfully built and tagged.",
                image_tag
            ));
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

    #[test]
    fn test_validate_image_tag_valid_inputs() {
        assert!(validate_image_tag("my-app:latest").is_ok());
        assert!(validate_image_tag("registry.example.com/team/app:v1.0.0").is_ok());
        assert!(validate_image_tag("app_web:sha-abcdef123").is_ok());
    }

    #[test]
    fn test_validate_image_tag_rejects_dangerous_inputs() {
        assert!(validate_image_tag("").is_err());
        assert!(validate_image_tag("-privileged").is_err());
        assert!(validate_image_tag("--help").is_err());
        assert!(validate_image_tag("app; rm -rf /").is_err());
        assert!(validate_image_tag("app $(whoami)").is_err());
        assert!(validate_image_tag("app\ntag").is_err());
    }

    #[tokio::test]
    async fn test_git_source_rejects_leading_dash() {
        let res = NixpacksBuilder::plan("--upload-pack=exploit").await;
        assert!(res.is_err());
    }
}
