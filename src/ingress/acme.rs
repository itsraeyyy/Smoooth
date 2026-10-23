use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::error::{Result, SmooothError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcmeDomain {
    pub main: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sans: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AcmeCertificateEntry {
    pub domain: AcmeDomain,
    pub certificate: String,
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "Store")]
    pub store: Option<String>,
}

pub struct AcmeKeystoreManager;

impl AcmeKeystoreManager {
    /// Parse a Traefik acme.json content and extract all certificate entries.
    /// If resolver is None, attempts to detect standard resolvers like "letsencrypt" or the first available key.
    pub fn parse_traefik_acme(
        content: &str,
        resolver: Option<&str>,
    ) -> Result<Vec<AcmeCertificateEntry>> {
        let root: Value = serde_json::from_str(content).map_err(|e| {
            SmooothError::Validation(format!("Invalid Traefik acme.json format: {}", e))
        })?;

        let resolver_obj = if let Some(res_name) = resolver {
            root.get(res_name).ok_or_else(|| {
                SmooothError::NotFound(format!("Resolver '{}' not found in acme.json", res_name))
            })?
        } else if let Some(le) = root.get("letsencrypt") {
            le
        } else if let Some(map) = root.as_object() {
            // Pick first resolver object that contains "Certificates"
            map.values()
                .find(|v| v.get("Certificates").is_some())
                .ok_or_else(|| {
                    SmooothError::NotFound("No ACME resolver found with Certificates in acme.json".to_string())
                })?
        } else {
            return Err(SmooothError::Validation(
                "Root of acme.json must be a JSON object".to_string(),
            ));
        };

        let certs_val = match resolver_obj.get("Certificates") {
            Some(Value::Array(arr)) => arr,
            Some(Value::Null) | None => return Ok(Vec::new()),
            _ => {
                return Err(SmooothError::Validation(
                    "Certificates field in acme.json must be an array or null".to_string(),
                ))
            }
        };

        let mut certificates = Vec::new();
        for item in certs_val {
            let entry: AcmeCertificateEntry = serde_json::from_value(item.clone()).map_err(|e| {
                SmooothError::Validation(format!("Failed to parse certificate entry: {}", e))
            })?;
            certificates.push(entry);
        }

        Ok(certificates)
    }

    /// Merge source certificates into target acme.json file idempotently.
    /// Preserves existing account data, existing certs, and sets strict 0600 permissions.
    /// Returns the number of newly added certificates.
    pub fn merge_into_traefik_acme(
        source_certs: Vec<AcmeCertificateEntry>,
        target_path: &Path,
        resolver: Option<&str>,
    ) -> Result<usize> {
        let res_name = resolver.unwrap_or("letsencrypt");

        let mut root: Value = if target_path.exists() {
            let content = fs::read_to_string(target_path).map_err(|e| {
                SmooothError::Storage(format!("Failed to read target acme.json: {}", e))
            })?;
            serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        if !root.is_object() {
            root = serde_json::json!({});
        }

        let root_map = root.as_object_mut().unwrap();
        if !root_map.contains_key(res_name) {
            root_map.insert(
                res_name.to_string(),
                serde_json::json!({ "Certificates": [] }),
            );
        }

        let resolver_map = root_map.get_mut(res_name).unwrap().as_object_mut().ok_or_else(|| {
            SmooothError::Validation(format!("Resolver '{}' in target must be an object", res_name))
        })?;

        let mut existing_certs: Vec<AcmeCertificateEntry> =
            if let Some(certs_arr) = resolver_map.get("Certificates").and_then(|v| v.as_array()) {
                certs_arr
                    .iter()
                    .filter_map(|c| serde_json::from_value(c.clone()).ok())
                    .collect()
            } else {
                Vec::new()
            };

        let mut known_domains: HashSet<String> = existing_certs
            .iter()
            .map(|c| c.domain.main.trim().to_lowercase())
            .collect();

        let mut added_count = 0;
        for sc in source_certs {
            let main_clean = sc.domain.main.trim().to_lowercase();
            if !known_domains.contains(&main_clean) {
                known_domains.insert(main_clean);
                existing_certs.push(sc);
                added_count += 1;
            }
        }

        resolver_map.insert(
            "Certificates".to_string(),
            serde_json::to_value(&existing_certs).map_err(|e| {
                SmooothError::Internal(anyhow::anyhow!("Serialization error: {}", e))
            })?,
        );

        let final_json = serde_json::to_string_pretty(&root).map_err(|e| {
            SmooothError::Internal(anyhow::anyhow!("Failed to format acme.json: {}", e))
        })?;

        Self::write_secure_acme(target_path, &final_json)?;

        Ok(added_count)
    }

    /// Write file with strict 0600 permissions required by Traefik and security policies
    pub fn write_secure_acme(path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, content).map_err(|e| {
            SmooothError::Storage(format!("Failed to write temporary acme file: {}", e))
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&temp_path)
                .map_err(|e| SmooothError::Storage(e.to_string()))?
                .permissions();
            perms.set_mode(0o600);
            let _ = fs::set_permissions(&temp_path, perms);
        }

        fs::rename(&temp_path, path).map_err(|e| {
            SmooothError::Storage(format!("Failed to atomically rename acme file: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    const SAMPLE_ACME: &str = r#"{
  "letsencrypt": {
    "Account": {
      "Email": "admin@example.com",
      "PrivateKey": "secret_key"
    },
    "Certificates": [
      {
        "domain": {
          "main": "app.example.com",
          "sans": ["www.app.example.com"]
        },
        "certificate": "CERT_BASE64_1",
        "key": "KEY_BASE64_1"
      },
      {
        "domain": {
          "main": "api.example.com"
        },
        "certificate": "CERT_BASE64_2",
        "key": "KEY_BASE64_2"
      }
    ]
  }
}"#;

    #[test]
    fn test_parse_traefik_acme() {
        let certs = AcmeKeystoreManager::parse_traefik_acme(SAMPLE_ACME, None).unwrap();
        assert_eq!(certs.len(), 2);
        assert_eq!(certs[0].domain.main, "app.example.com");
        assert_eq!(certs[0].domain.sans.as_ref().unwrap().len(), 1);
        assert_eq!(certs[1].domain.main, "api.example.com");
    }

    #[test]
    fn test_merge_into_traefik_acme_deduplication() {
        let temp_dir = std::env::temp_dir().join(format!("acme_test_{}", Uuid::new_v4().simple()));
        let target_path = temp_dir.join("acme.json");

        // First pass: add 2 certs
        let source_certs = AcmeKeystoreManager::parse_traefik_acme(SAMPLE_ACME, None).unwrap();
        let added = AcmeKeystoreManager::merge_into_traefik_acme(source_certs.clone(), &target_path, None).unwrap();
        assert_eq!(added, 2);

        // Second pass with duplicate certs: should add 0
        let added_again = AcmeKeystoreManager::merge_into_traefik_acme(source_certs, &target_path, None).unwrap();
        assert_eq!(added_again, 0);

        // Add 1 new cert
        let new_cert = vec![AcmeCertificateEntry {
            domain: AcmeDomain {
                main: "new.example.com".to_string(),
                sans: None,
            },
            certificate: "NEW_CERT".to_string(),
            key: "NEW_KEY".to_string(),
            store: None,
        }];
        let added_third = AcmeKeystoreManager::merge_into_traefik_acme(new_cert, &target_path, None).unwrap();
        assert_eq!(added_third, 1);

        // Verify final count
        let final_content = fs::read_to_string(&target_path).unwrap();
        let final_certs = AcmeKeystoreManager::parse_traefik_acme(&final_content, None).unwrap();
        assert_eq!(final_certs.len(), 3);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
