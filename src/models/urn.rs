use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::{Result, SmooothError};

/// Structured, canonical resource identifier.
///
/// Follows the grammar:
/// `smoooth://workspaces/<workspace>[/<type>/<id>...]`
///
/// Guaranteed Invariants:
/// - Cannot be constructed in an invalid state. Fields are private.
/// - Deserialization and parsing enforce identical strict canonical rules.
/// - Identifiers preserve casing for resource IDs, but reject path traversals,
///   control characters, colons, slashes, and malformed symbols.
/// - Valid parent-child hierarchies are enforced at parse/construction time.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceUrn {
    workspace: String,
    segments: Vec<UrnSegment>,
}

/// Typed path segments in the resource hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "snake_case")]
pub enum UrnSegment {
    Workload(String),
    Database(String),
    Network(String),
    Volume(String),
    Secret(String),
    Ingress(String),
    Logs(String),
    Snapshot(String),
}

impl UrnSegment {
    pub fn workload(id: &str) -> Result<Self> {
        validate_resource_identifier(id)?;
        Ok(Self::Workload(id.to_string()))
    }

    pub fn database(id: &str) -> Result<Self> {
        validate_resource_identifier(id)?;
        Ok(Self::Database(id.to_string()))
    }

    pub fn network(id: &str) -> Result<Self> {
        validate_resource_identifier(id)?;
        Ok(Self::Network(id.to_string()))
    }

    pub fn volume(id: &str) -> Result<Self> {
        validate_resource_identifier(id)?;
        Ok(Self::Volume(id.to_string()))
    }

    pub fn secret(id: &str) -> Result<Self> {
        validate_resource_identifier(id)?;
        Ok(Self::Secret(id.to_string()))
    }

    pub fn ingress(id: &str) -> Result<Self> {
        validate_resource_identifier(id)?;
        Ok(Self::Ingress(id.to_string()))
    }

    pub fn logs(id: &str) -> Result<Self> {
        validate_resource_identifier(id)?;
        Ok(Self::Logs(id.to_string()))
    }

    pub fn snapshot(id: &str) -> Result<Self> {
        validate_resource_identifier(id)?;
        Ok(Self::Snapshot(id.to_string()))
    }

    pub fn segment_type(&self) -> &'static str {
        match self {
            UrnSegment::Workload(_) => "workloads",
            UrnSegment::Database(_) => "databases",
            UrnSegment::Network(_) => "networks",
            UrnSegment::Volume(_) => "volumes",
            UrnSegment::Secret(_) => "secrets",
            UrnSegment::Ingress(_) => "ingresses",
            UrnSegment::Logs(_) => "logs",
            UrnSegment::Snapshot(_) => "snapshots",
        }
    }

    pub fn id(&self) -> &str {
        match self {
            UrnSegment::Workload(id) => id,
            UrnSegment::Database(id) => id,
            UrnSegment::Network(id) => id,
            UrnSegment::Volume(id) => id,
            UrnSegment::Secret(id) => id,
            UrnSegment::Ingress(id) => id,
            UrnSegment::Logs(id) => id,
            UrnSegment::Snapshot(id) => id,
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            UrnSegment::Workload(_) => "Workload",
            UrnSegment::Database(_) => "Database",
            UrnSegment::Network(_) => "Network",
            UrnSegment::Volume(_) => "Volume",
            UrnSegment::Secret(_) => "Secret",
            UrnSegment::Ingress(_) => "Ingress",
            UrnSegment::Logs(_) => "Logs",
            UrnSegment::Snapshot(_) => "Snapshot",
        }
    }
}

/// Enforces structural resource hierarchy rules.
///
/// Rules:
/// - Under a workspace (parent is None): Workload, Database, Network, Volume, Secret, and Ingress are allowed.
///   Logs and Snapshot cannot be direct children of a workspace.
/// - Under a Workload: Logs, Secret, and Snapshot are allowed.
/// - Under a Database: Secret, Snapshot, and Logs are allowed.
/// - Under a Volume: Snapshot is allowed.
/// - Secret, Logs, Snapshot, Network, Ingress: Leaf resources; cannot have children.
pub fn validate_hierarchy(parent: Option<&UrnSegment>, child: &UrnSegment) -> Result<()> {
    match parent {
        None => match child {
            UrnSegment::Workload(_)
            | UrnSegment::Database(_)
            | UrnSegment::Network(_)
            | UrnSegment::Volume(_)
            | UrnSegment::Secret(_)
            | UrnSegment::Ingress(_) => Ok(()),
            UrnSegment::Logs(_) => Err(SmooothError::InvalidResourceUrn(
                "Hierarchy violation: 'logs' cannot be placed directly under workspace; must be a child of 'workloads' or 'databases'".into(),
            )),
            UrnSegment::Snapshot(_) => Err(SmooothError::InvalidResourceUrn(
                "Hierarchy violation: 'snapshots' cannot be placed directly under workspace; must be a child of 'databases' or 'volumes'".into(),
            )),
        },
        Some(UrnSegment::Workload(_)) => match child {
            UrnSegment::Logs(_) | UrnSegment::Secret(_) | UrnSegment::Snapshot(_) => Ok(()),
            other => Err(SmooothError::InvalidResourceUrn(format!(
                "Hierarchy violation: 'workloads' cannot own child of type '{}'",
                other.segment_type()
            ))),
        },
        Some(UrnSegment::Database(_)) => match child {
            UrnSegment::Secret(_) | UrnSegment::Snapshot(_) | UrnSegment::Logs(_) => Ok(()),
            other => Err(SmooothError::InvalidResourceUrn(format!(
                "Hierarchy violation: 'databases' cannot own child of type '{}'",
                other.segment_type()
            ))),
        },
        Some(UrnSegment::Volume(_)) => match child {
            UrnSegment::Snapshot(_) => Ok(()),
            other => Err(SmooothError::InvalidResourceUrn(format!(
                "Hierarchy violation: 'volumes' cannot own child of type '{}'",
                other.segment_type()
            ))),
        },
        Some(leaf) => Err(SmooothError::InvalidResourceUrn(format!(
            "Hierarchy violation: '{}' is a leaf resource and cannot own child '{}'",
            leaf.segment_type(),
            child.segment_type()
        ))),
    }
}

impl ResourceUrn {
    /// Strict parser for canonical Resource URNs.
    ///
    /// Grammar: `smoooth://workspaces/<workspace>[/<type>/<id>...]`
    pub fn parse(input: &str) -> Result<Self> {
        let scheme = "smoooth://workspaces/";
        if !input.starts_with(scheme) {
            return Err(SmooothError::InvalidResourceUrn(format!(
                "URN must start with '{scheme}', got: '{input}'"
            )));
        }

        let remainder = &input[scheme.len()..];
        if remainder.is_empty() {
            return Err(SmooothError::InvalidResourceUrn(
                "URN is missing workspace identifier".into(),
            ));
        }

        if remainder.ends_with('/') {
            return Err(SmooothError::InvalidResourceUrn(
                "URN must not have a trailing slash".into(),
            ));
        }

        if remainder.contains("//") {
            return Err(SmooothError::InvalidResourceUrn(
                "URN must not contain empty path segments ('//')".into(),
            ));
        }

        let parts: Vec<&str> = remainder.split('/').collect();
        let workspace = parts[0];
        validate_workspace_identifier(workspace)?;

        let mut segments = Vec::new();
        let mut idx = 1;
        let mut current_parent: Option<&UrnSegment> = None;

        while idx < parts.len() {
            let seg_type = parts[idx];
            if idx + 1 >= parts.len() {
                return Err(SmooothError::InvalidResourceUrn(format!(
                    "Incomplete URN path: segment type '{seg_type}' is missing its identifier"
                )));
            }
            let seg_id = parts[idx + 1];
            validate_resource_identifier(seg_id)?;

            let segment = match seg_type {
                "workloads" => UrnSegment::Workload(seg_id.to_string()),
                "databases" => UrnSegment::Database(seg_id.to_string()),
                "networks" => UrnSegment::Network(seg_id.to_string()),
                "volumes" => UrnSegment::Volume(seg_id.to_string()),
                "secrets" => UrnSegment::Secret(seg_id.to_string()),
                "ingresses" => UrnSegment::Ingress(seg_id.to_string()),
                "logs" => UrnSegment::Logs(seg_id.to_string()),
                "snapshots" => UrnSegment::Snapshot(seg_id.to_string()),
                unknown => {
                    return Err(SmooothError::InvalidResourceUrn(format!(
                        "Unknown resource segment type: '{unknown}'"
                    )))
                }
            };

            validate_hierarchy(current_parent, &segment)?;

            segments.push(segment);
            current_parent = segments.last();
            idx += 2;
        }

        Ok(Self {
            workspace: workspace.to_string(),
            segments,
        })
    }

    /// Construct a root workspace URN.
    pub fn new_workspace(workspace: &str) -> Result<Self> {
        validate_workspace_identifier(workspace)?;
        Ok(Self {
            workspace: workspace.to_string(),
            segments: Vec::new(),
        })
    }

    /// Construct a new child URN by appending a valid segment.
    /// Rejects nonsensical hierarchies.
    pub fn child(&self, segment: UrnSegment) -> Result<Self> {
        validate_hierarchy(self.segments.last(), &segment)?;
        let mut new_segments = self.segments.clone();
        new_segments.push(segment);
        Ok(Self {
            workspace: self.workspace.clone(),
            segments: new_segments,
        })
    }

    /// Workspace name accessor.
    pub fn workspace(&self) -> &str {
        &self.workspace
    }

    /// Path segments accessor.
    pub fn segments(&self) -> &[UrnSegment] {
        &self.segments
    }

    /// Returns true if this URN represents a workspace root with no sub-resources.
    pub fn is_workspace_root(&self) -> bool {
        self.segments.is_empty()
    }

    /// Evaluates hierarchical containment: returns true if `self` encompasses `target`.
    ///
    /// Rules:
    /// - Workspaces must match exactly.
    /// - If `self` has no segments (workspace root), it encompasses all resources in that workspace.
    /// - If `self` has more segments than `target`, it cannot encompass `target`.
    /// - Every segment in `self` must match the corresponding segment in `target` at the same index.
    pub fn encompasses(&self, target: &ResourceUrn) -> bool {
        if self.workspace != target.workspace {
            return false;
        }
        if self.segments.is_empty() {
            return true;
        }
        if self.segments.len() > target.segments.len() {
            return false;
        }
        for (i, seg) in self.segments.iter().enumerate() {
            if seg != &target.segments[i] {
                return false;
            }
        }
        true
    }

    /// Formats the URN into its canonical string representation.
    pub fn to_canonical_string(&self) -> String {
        let mut out = format!("smoooth://workspaces/{}", self.workspace);
        for seg in &self.segments {
            out.push('/');
            out.push_str(seg.segment_type());
            out.push('/');
            out.push_str(seg.id());
        }
        out
    }
}

fn validate_workspace_identifier(ws: &str) -> Result<()> {
    if ws.is_empty() || ws.len() > 64 {
        return Err(SmooothError::InvalidResourceUrn(
            "Workspace identifier must be between 1 and 64 characters".into(),
        ));
    }
    if ws.contains("..") || ws.contains('/') || ws.contains('\\') || ws.contains('\0') {
        return Err(SmooothError::InvalidResourceUrn(
            "Workspace identifier contains prohibited path traversal characters".into(),
        ));
    }
    let bytes = ws.as_bytes();
    if bytes[0] == b'-' || bytes[bytes.len() - 1] == b'-' {
        return Err(SmooothError::InvalidResourceUrn(
            "Workspace identifier cannot start or end with a hyphen".into(),
        ));
    }
    let valid = ws.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !valid {
        return Err(SmooothError::InvalidResourceUrn(format!(
            "Workspace identifier '{ws}' must be strictly lowercase alphanumeric and hyphens"
        )));
    }
    Ok(())
}

fn validate_resource_identifier(id: &str) -> Result<()> {
    if id.is_empty() || id.len() > 64 {
        return Err(SmooothError::InvalidResourceUrn(
            "Resource identifier must be between 1 and 64 characters".into(),
        ));
    }
    if id.contains("..") || id.contains('/') || id.contains('\\') || id.contains('\0') {
        return Err(SmooothError::InvalidResourceUrn(
            "Resource identifier contains prohibited path traversal characters".into(),
        ));
    }
    let bytes = id.as_bytes();
    if bytes[0] == b'-' || bytes[0] == b'.' {
        return Err(SmooothError::InvalidResourceUrn(
            "Resource identifier cannot start with a hyphen or dot".into(),
        ));
    }
    let valid = id.chars().all(|c| {
        c.is_ascii_lowercase()
            || c.is_ascii_uppercase()
            || c.is_ascii_digit()
            || c == '-'
            || c == '_'
            || c == '.'
    });
    if !valid {
        return Err(SmooothError::InvalidResourceUrn(format!(
            "Resource identifier '{id}' contains invalid characters (allowed: [a-zA-Z0-9-_.]): "
        )));
    }
    Ok(())
}

impl fmt::Display for ResourceUrn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_canonical_string())
    }
}

impl FromStr for ResourceUrn {
    type Err = SmooothError;
    fn from_str(s: &str) -> Result<Self> {
        ResourceUrn::parse(s)
    }
}

impl TryFrom<&str> for ResourceUrn {
    type Error = SmooothError;
    fn try_from(s: &str) -> Result<Self> {
        ResourceUrn::parse(s)
    }
}

impl TryFrom<String> for ResourceUrn {
    type Error = SmooothError;
    fn try_from(s: String) -> Result<Self> {
        ResourceUrn::parse(&s)
    }
}

impl Serialize for ResourceUrn {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_canonical_string())
    }
}

impl<'de> Deserialize<'de> for ResourceUrn {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ResourceUrn::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_urn_parsing_and_canonical_string() {
        let valid_cases = vec![
            "smoooth://workspaces/production",
            "smoooth://workspaces/production/workloads/web-api",
            "smoooth://workspaces/production/databases/postgres-primary",
            "smoooth://workspaces/production/databases/postgres-primary/secrets/db_password",
            "smoooth://workspaces/production/databases/postgres-primary/snapshots/backup_20261031",
            "smoooth://workspaces/production/workloads/web-api/logs/stdout",
            "smoooth://workspaces/production/workloads/web-api/secrets/jwt-token",
            "smoooth://workspaces/production/workloads/web-api/snapshots/v1.0.0",
            "smoooth://workspaces/production/volumes/data-vol/snapshots/snap-1",
            "smoooth://workspaces/staging/networks/app-bridge",
            "smoooth://workspaces/default/ingresses/main-route",
            "smoooth://workspaces/default/secrets/global-api-key",
        ];

        for case in valid_cases {
            let parsed = ResourceUrn::parse(case).expect(&format!("Failed to parse: {case}"));
            assert_eq!(parsed.to_canonical_string(), case);
            assert_eq!(format!("{parsed}"), case);

            // Test roundtrip serde JSON
            let json = serde_json::to_string(&parsed).expect("Serialize to JSON");
            assert_eq!(json, format!("\"{case}\""));
            let deserialized: ResourceUrn = serde_json::from_str(&json).expect("Deserialize JSON");
            assert_eq!(deserialized, parsed);
        }
    }

    #[test]
    fn test_resource_id_case_preservation() {
        let urn_str = "smoooth://workspaces/prod/workloads/MyAwesomeAPI";
        let parsed = ResourceUrn::parse(urn_str).expect("Case-preserved parse");
        assert_eq!(parsed.segments()[0].id(), "MyAwesomeAPI");
        assert_eq!(parsed.to_canonical_string(), urn_str);
    }

    #[test]
    fn test_invalid_workspace_identifiers_rejected() {
        let invalid_workspaces = vec![
            "smoooth://workspaces/Prod",              // uppercase forbidden in workspace
            "smoooth://workspaces/-prod",             // leading hyphen
            "smoooth://workspaces/prod-",             // trailing hyphen
            "smoooth://workspaces/prod_test",         // underscore forbidden in workspace
            "smoooth://workspaces/prod..staging",     // path traversal
            "smoooth://workspaces/prod/sub",          // slash without segment type
            "smoooth://workspaces/",                  // empty workspace
            "smoooth://workspaces/prod space",        // spaces
            "smoooth://workspaces/prod:test",         // colons
        ];

        for case in invalid_workspaces {
            let res = ResourceUrn::parse(case);
            assert!(res.is_err(), "Expected failure for invalid workspace: {case}");
        }
    }

    #[test]
    fn test_invalid_resource_identifiers_rejected() {
        let invalid_ids = vec![
            "smoooth://workspaces/prod/workloads/-leading-hyphen",
            "smoooth://workspaces/prod/workloads/.leading-dot",
            "smoooth://workspaces/prod/workloads/../traversal",
            "smoooth://workspaces/prod/workloads/has space",
            "smoooth://workspaces/prod/workloads/colons:bad",
            "smoooth://workspaces/prod/workloads/semi;bad",
            "smoooth://workspaces/prod/workloads/",
        ];

        for case in invalid_ids {
            let res = ResourceUrn::parse(case);
            assert!(res.is_err(), "Expected failure for invalid resource ID: {case}");
        }
    }

    #[test]
    fn test_invalid_hierarchies_rejected() {
        let invalid_hierarchies = vec![
            // Logs cannot be direct child of workspace
            "smoooth://workspaces/prod/logs/stdout",
            // Snapshots cannot be direct child of workspace
            "smoooth://workspaces/prod/snapshots/snap1",
            // Workload cannot own a Database
            "smoooth://workspaces/prod/workloads/web/databases/db",
            // Workload cannot own another Workload
            "smoooth://workspaces/prod/workloads/web/workloads/child",
            // Database cannot own a Workload
            "smoooth://workspaces/prod/databases/db/workloads/web",
            // Secrets are leaf nodes; cannot own a Workload
            "smoooth://workspaces/prod/secrets/my-sec/workloads/web",
            // Secrets cannot own Secrets
            "smoooth://workspaces/prod/secrets/sec1/secrets/sec2",
            // Logs are leaf nodes; cannot own Snapshots
            "smoooth://workspaces/prod/workloads/web/logs/stdout/snapshots/snap1",
            // Volume cannot own a Workload
            "smoooth://workspaces/prod/volumes/vol1/workloads/web",
            // Network is a leaf node; cannot own Workload
            "smoooth://workspaces/prod/networks/net1/workloads/web",
            // Ingress is a leaf node; cannot own anything
            "smoooth://workspaces/prod/ingresses/ing1/secrets/cert",
        ];

        for case in invalid_hierarchies {
            let res = ResourceUrn::parse(case);
            assert!(res.is_err(), "Expected hierarchy rejection for: {case}");
            if let Err(SmooothError::InvalidResourceUrn(msg)) = res {
                assert!(msg.contains("Hierarchy violation"), "Message should mention hierarchy violation: {msg}");
            } else {
                panic!("Expected InvalidResourceUrn error variant");
            }
        }
    }

    #[test]
    fn test_deserialization_rejects_malformed_input() {
        // Deserializing invalid hierarchy via JSON
        let malformed_json = "\"smoooth://workspaces/prod/secrets/sec1/workloads/web\"";
        let res: std::result::Result<ResourceUrn, _> = serde_json::from_str(malformed_json);
        assert!(res.is_err(), "Deserializing invalid hierarchy must fail");

        // Deserializing non-string JSON (e.g. attempted struct bypass)
        let struct_bypass_json = "{\"workspace\":\"prod\",\"segments\":[]}";
        let res2: std::result::Result<ResourceUrn, _> = serde_json::from_str(struct_bypass_json);
        assert!(res2.is_err(), "Deserializing struct JSON must fail");

        // Deserializing path traversal
        let traversal_json = "\"smoooth://workspaces/prod/workloads/..%2F..%2Fetc\"";
        let res3: std::result::Result<ResourceUrn, _> = serde_json::from_str(traversal_json);
        assert!(res3.is_err(), "Deserializing traversal must fail");
    }

    #[test]
    fn test_hierarchical_containment_encompasses() {
        let ws_root = ResourceUrn::parse("smoooth://workspaces/production").unwrap();
        let staging_root = ResourceUrn::parse("smoooth://workspaces/staging").unwrap();
        let workload_web = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api").unwrap();
        let workload_web_prefix = ResourceUrn::parse("smoooth://workspaces/production/workloads/web").unwrap();
        let workload_logs = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api/logs/stdout").unwrap();
        let workload_secret = ResourceUrn::parse("smoooth://workspaces/production/workloads/web-api/secrets/jwt").unwrap();
        let db_prod = ResourceUrn::parse("smoooth://workspaces/production/databases/postgres").unwrap();

        // 1. Workspace root encompasses child workloads and databases in same workspace
        assert!(ws_root.encompasses(&workload_web));
        assert!(ws_root.encompasses(&workload_logs));
        assert!(ws_root.encompasses(&db_prod));

        // 2. Different workspace does not encompass
        assert!(!staging_root.encompasses(&workload_web));
        assert!(!ws_root.encompasses(&staging_root));

        // 3. Workload encompasses its own subresources
        assert!(workload_web.encompasses(&workload_logs));
        assert!(workload_web.encompasses(&workload_secret));

        // 4. Subresource does not encompass parent
        assert!(!workload_logs.encompasses(&workload_web));

        // 5. Distinct workloads do NOT encompass each other even if one is string prefix of another!
        // (web vs web-api)
        assert!(!workload_web_prefix.encompasses(&workload_web));
        assert!(!workload_web.encompasses(&workload_web_prefix));

        // 6. Workload does not encompass Database
        assert!(!workload_web.encompasses(&db_prod));
    }

    #[test]
    fn test_child_builder_respects_hierarchy() {
        let ws = ResourceUrn::new_workspace("production").unwrap();
        let wl = ws.child(UrnSegment::Workload("web-api".into())).unwrap();
        assert_eq!(wl.to_canonical_string(), "smoooth://workspaces/production/workloads/web-api");

        let logs = wl.child(UrnSegment::Logs("stdout".into())).unwrap();
        assert_eq!(logs.to_canonical_string(), "smoooth://workspaces/production/workloads/web-api/logs/stdout");

        // Attempting to add child to leaf logs must fail
        let invalid_child = logs.child(UrnSegment::Snapshot("snap".into()));
        assert!(invalid_child.is_err());
    }
}
