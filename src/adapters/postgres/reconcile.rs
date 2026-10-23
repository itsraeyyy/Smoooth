use serde::{Deserialize, Serialize};
use tokio_postgres::Client;

use crate::error::{Result, SmooothError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupabaseReconciliationReport {
    pub detected_supabase: bool,
    pub roles_updated: Vec<String>,
    pub functions_reassigned: Vec<String>,
    pub schemas_reassigned: Vec<String>,
    pub success: bool,
}

pub struct SupabaseRoleReconciler;

impl SupabaseRoleReconciler {
    /// Detect if the connected database has Supabase schemas or roles
    pub async fn is_supabase_instance(client: &Client) -> Result<bool> {
        let query = "
            SELECT count(*) 
            FROM information_schema.schemata 
            WHERE schema_name IN ('auth', 'storage', 'vault');
        ";
        let row = client
            .query_one(query, &[])
            .await
            .map_err(|e| SmooothError::PostgresFailed(format!("Failed to query schemata: {}", e)))?;

        let schema_count: i64 = row.get(0);
        if schema_count > 0 {
            return Ok(true);
        }

        let role_query = "
            SELECT count(*) 
            FROM pg_roles 
            WHERE rolname IN ('supabase_admin', 'supabase_auth_admin', 'supabase_storage_admin');
        ";
        let role_row = client
            .query_one(role_query, &[])
            .await
            .map_err(|e| SmooothError::PostgresFailed(format!("Failed to query roles: {}", e)))?;

        let role_count: i64 = role_row.get(0);
        Ok(role_count > 0)
    }

    /// Reconcile Supabase internal roles, function ownership, and schema permissions
    pub async fn reconcile_roles(
        client: &Client,
        target_password: &str,
    ) -> Result<SupabaseReconciliationReport> {
        let is_supabase = Self::is_supabase_instance(client).await?;
        if !is_supabase {
            return Ok(SupabaseReconciliationReport {
                detected_supabase: false,
                roles_updated: Vec::new(),
                functions_reassigned: Vec::new(),
                schemas_reassigned: Vec::new(),
                success: true,
            });
        }

        let mut roles_updated = Vec::new();
        let mut functions_reassigned = Vec::new();
        let mut schemas_reassigned = Vec::new();

        // 1. Grant postgres to supabase_admin if both exist
        let grant_sql = "
            DO $$
            BEGIN
              IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'supabase_admin') 
                 AND EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'postgres') THEN
                GRANT postgres TO supabase_admin;
              END IF;
            END
            $$;
        ";
        client
            .batch_execute(grant_sql)
            .await
            .map_err(|e| SmooothError::PostgresFailed(format!("Failed to grant postgres role: {}", e)))?;

        // 2. Align role passwords
        let target_roles = [
            "supabase_auth_admin",
            "supabase_storage_admin",
            "supabase_admin",
            "postgres",
        ];

        for role in target_roles {
            let check_sql = "SELECT 1 FROM pg_roles WHERE rolname = $1;";
            let rows = client
                .query(check_sql, &[&role])
                .await
                .map_err(|e| SmooothError::PostgresFailed(format!("Failed checking role '{}': {}", role, e)))?;

            if !rows.is_empty() {
                // Alter password
                let alter_sql = format!("ALTER ROLE {} WITH PASSWORD $1;", role);
                client
                    .execute(&alter_sql, &[&target_password])
                    .await
                    .map_err(|e| {
                        SmooothError::PostgresFailed(format!("Failed to alter password for role '{}': {}", role, e))
                    })?;
                roles_updated.push(role.to_string());
            }
        }

        // 3. Reconcile function ownership for auth
        let auth_funcs = ["uid", "role", "email"];
        for func in auth_funcs {
            let func_check = "
                SELECT 1 
                FROM pg_proc p 
                JOIN pg_namespace n ON p.pronamespace = n.oid 
                WHERE n.nspname = 'auth' AND p.proname = $1;
            ";
            let rows = client
                .query(func_check, &[&func])
                .await
                .map_err(|e| SmooothError::PostgresFailed(format!("Failed checking function 'auth.{}': {}", func, e)))?;

            if !rows.is_empty() {
                let alter_func_sql = format!("ALTER FUNCTION auth.{}() OWNER TO supabase_auth_admin;", func);
                if client.batch_execute(&alter_func_sql).await.is_ok() {
                    functions_reassigned.push(format!("auth.{}()", func));
                }
            }
        }

        // 4. Storage schema privileges and ownership
        let storage_check = "SELECT 1 FROM information_schema.schemata WHERE schema_name = 'storage';";
        let storage_rows = client
            .query(storage_check, &[])
            .await
            .map_err(|e| SmooothError::PostgresFailed(format!("Failed checking storage schema: {}", e)))?;

        if !storage_rows.is_empty() {
            let storage_sql = "
                DO $$
                BEGIN
                  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'supabase_storage_admin') THEN
                    GRANT ALL ON SCHEMA storage TO supabase_storage_admin;
                    ALTER SCHEMA storage OWNER TO supabase_storage_admin;
                  END IF;
                END
                $$;
            ";
            if client.batch_execute(storage_sql).await.is_ok() {
                schemas_reassigned.push("storage".to_string());
            }
        }

        Ok(SupabaseReconciliationReport {
            detected_supabase: true,
            roles_updated,
            functions_reassigned,
            schemas_reassigned,
            success: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_serialization() {
        let report = SupabaseReconciliationReport {
            detected_supabase: true,
            roles_updated: vec!["supabase_auth_admin".to_string(), "supabase_storage_admin".to_string()],
            functions_reassigned: vec!["auth.uid()".to_string()],
            schemas_reassigned: vec!["storage".to_string()],
            success: true,
        };

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("supabase_auth_admin"));
        assert!(json.contains("auth.uid()"));
    }
}
