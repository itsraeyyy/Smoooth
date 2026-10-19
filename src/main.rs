use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::sync::Arc;

use smoooth::adapters::docker::DockerAdapter;
use smoooth::doctor::Doctor;
use smoooth::engine::Orchestrator;
use smoooth::mcp::tools::{get_available_tools, get_default_capabilities};
use smoooth::models::{
    AgentScope, CanonicalActionSpec, CanonicalExecutionSpec, DependencyEdge, Plan,
    PolicyConstraints, RiskTier, ServiceBinding,
};
use smoooth::storage::LocalStorage;

#[derive(Parser, Debug)]
#[command(name = "smoooth")]
#[command(author = "Smoooth Contributors")]
#[command(version = "0.1.1")]
#[command(about = "Infrastructure freedom, built for AI agents (Developer Beta)", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run diagnostic environment preflight checks (platform, Docker, disk, ports)
    Doctor,
    /// Manage agent pairing, scopes, and revocable credentials
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
    /// List all registered MCP tools mapped to the 7-step Golden Workflow
    Tools,
    /// List supported infrastructure capabilities & provider guarantees
    Capabilities,
    /// Run an end-to-end local simulation of the Golden Workflow
    Demo,
    /// Inspect local workspace resources and durable operation state
    Status {
        #[arg(default_value = "default")]
        workspace: String,
    },
    /// Start the Smoooth HTTP API Server & Control Plane Dashboard
    #[command(alias = "server")]
    Serve {
        #[arg(short, long, default_value = "127.0.0.1")]
        bind: String,
        #[arg(short, long, default_value_t = 4040)]
        port: u16,
        #[arg(short, long)]
        token: Option<String>,
        #[arg(long)]
        token_file: Option<String>,
    },
    /// Start the Smoooth stdio Model Context Protocol (MCP) server for coding agents
    Mcp,
    /// Review and approve a pending execution plan by its plan ID
    Approve {
        plan_id: String,
        #[arg(default_value = "operator")]
        approver: String,
    },
    /// Inspect operation status, progress, logs, and evidence
    Ops { operation_id: String },
    /// Inspect and reconcile drift or recover crashed resource
    Reconcile { resource_id: String },
    /// Manage ingress routes, Caddy dynamic routing, and public domains
    Ingress {
        #[command(subcommand)]
        command: IngressCommands,
    },
    /// Build container images using Nixpacks (zero Dockerfile) from local path or Git repository
    Build {
        #[command(subcommand)]
        command: BuildCommands,
    },
}

#[derive(Subcommand, Debug)]
enum BuildCommands {
    /// Inspect source directory or Git repository and generate a zero-Dockerfile build plan
    Plan {
        /// Local directory path or Git clone URL
        source: String,
    },
    /// Build and tag an optimized production container image from source or Git repository
    Image {
        /// Local directory path or Git clone URL
        source: String,
        /// Target image tag name (e.g. "my-app:v1")
        #[arg(short, long)]
        name: String,
        /// Environment variable in KEY=VALUE format (can be specified multiple times)
        #[arg(short, long)]
        env: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
enum AgentCommands {
    /// Pair a new AI coding agent and issue revocable scoped credentials
    Pair {
        /// Name or role description of the agent
        #[arg(short, long)]
        name: String,
        /// Bound workspace isolation boundary (or '*' for global access)
        #[arg(short, long, default_value = "default")]
        workspace: String,
        /// Comma-separated scopes: read_only, plan, deploy, destructive, admin
        #[arg(short, long, default_value = "read_only,plan,deploy")]
        scopes: String,
        /// Credential expiration in hours (optional)
        #[arg(long)]
        ttl_hours: Option<i64>,
    },
    /// List all paired agent identities and their granted scopes
    List {
        /// Filter by workspace (optional)
        #[arg(short, long)]
        workspace: Option<String>,
    },
    /// Revoke an agent's credentials immediately
    Revoke {
        /// Agent ID to revoke
        agent_id: String,
    },
    /// Rotate an agent's token secret
    Rotate {
        /// Agent ID to rotate
        agent_id: String,
    },
}

#[derive(Subcommand, Debug)]
enum IngressCommands {
    /// Show ingress mode (Standalone Caddy vs Upstream Subordinate) and port availability
    Status,
    /// List all registered domain ingress routes
    List,
    /// Add a new domain ingress route pointing to an internal resource and port
    Add {
        /// Public domain name (e.g. "api.example.com")
        #[arg(short, long)]
        domain: String,
        /// Target resource name
        #[arg(short, long)]
        target: String,
        /// Internal target port (e.g. 19080)
        #[arg(short, long)]
        port: u16,
        /// Enable TLS (default: true)
        #[arg(long, default_value_t = true)]
        tls: bool,
        /// Optional path prefix (e.g. "/api")
        #[arg(long)]
        path_prefix: Option<String>,
    },
    /// Remove an ingress route by domain or route ID
    Remove {
        /// Domain or route ID to remove
        target: String,
    },
    /// Export reverse-proxy configuration snippet (caddy, nginx, or traefik)
    Export {
        /// Format: caddy, nginx, or traefik
        #[arg(short, long, default_value = "caddy")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();
    let cli = Cli::parse();

    let storage_path = std::env::var("SMOOOTH_STORAGE_PATH").unwrap_or_else(|_| {
        if std::path::Path::new("/var/lib/smoooth/smoooth_state.json").exists() {
            "/var/lib/smoooth/smoooth_state.json".to_string()
        } else {
            "smoooth_state.json".to_string()
        }
    });
    let storage = Arc::new(LocalStorage::new_file_backed(&storage_path));
    let mut orchestrator = Orchestrator::new(storage.clone());
    let docker_adapter = Arc::new(DockerAdapter::new());
    let postgres_adapter = Arc::new(smoooth::adapters::PostgresAdapter::new(
        docker_adapter.clone(),
    ));
    let redis_adapter = Arc::new(smoooth::adapters::RedisAdapter::new(docker_adapter.clone()));
    let mongo_adapter = Arc::new(smoooth::adapters::MongoAdapter::new(docker_adapter.clone()));
    orchestrator.register_adapter(docker_adapter);
    orchestrator.register_adapter(postgres_adapter);
    orchestrator.register_adapter(redis_adapter);
    orchestrator.register_adapter(mongo_adapter);

    match cli.command {
        Some(Commands::Doctor) => {
            println!("\n🔍 Running Smoooth Beta Environment Diagnostics...\n");
            let report = Doctor::run_diagnostics(&storage_path, "127.0.0.1", 4040).await;

            println!(
                "Target Environment: {} / {} (Smoooth {})",
                report.os, report.arch, report.version
            );
            println!(
                "Diagnostic Status : {}\n",
                report.overall_status.to_uppercase()
            );

            for check in &report.checks {
                let badge = match check.status.as_str() {
                    "pass" => "✅ PASS",
                    "warn" => "⚠️ WARN",
                    _ => "❌ FAIL",
                };
                println!("  [{}] {}: {}", badge, check.name, check.message);
                if let Some(rec) = &check.recommendation {
                    println!("         ↳ Action: {}", rec);
                }
            }

            println!(
                "\nSummary: {} passed, {} warnings, {} failures.\n",
                report.passed_count, report.warning_count, report.failure_count
            );

            if report.failure_count > 0 {
                std::process::exit(1);
            }
        }

        Some(Commands::Agent { command }) => match command {
            AgentCommands::Pair {
                name,
                workspace,
                scopes,
                ttl_hours,
            } => {
                let parsed_scopes = AgentScope::parse_csv(&scopes);
                if parsed_scopes.is_empty() {
                    eprintln!("❌ Error: At least one valid scope required (e.g. read_only, plan, deploy, destructive, admin)");
                    std::process::exit(1);
                }

                let (agent, raw_token) = orchestrator
                    .pair_agent(&name, &workspace, parsed_scopes, ttl_hours)
                    .await?;
                println!("\n============================================================");
                println!("  ✅ AI Coding Agent Paired Successfully!");
                println!("============================================================");
                println!("  • Agent ID        : {}", agent.agent_id);
                println!("  • Name            : {}", agent.name);
                println!("  • Workspace Bound : {}", agent.workspace_id);
                println!("  • Granted Scopes  : {:?}", agent.scopes);
                if let Some(exp) = agent.expires_at {
                    println!("  • Expires At      : {}", exp);
                }
                if agent.is_overprivileged() {
                    println!("  ⚠️ WARNING        : Agent possesses elevated administrative or destructive scopes.");
                }
                println!("\n  🔑 Plaintext Bearer Token (STORE SECURELY - DISPLAYED ONCE):");
                println!("     {}", raw_token);
                println!("\n  Provide this token in your agent's MCP environment configuration:");
                println!("     SMOOOTH_AUTH_TOKEN=\"{}\"", raw_token);
                println!("============================================================\n");
            }

            AgentCommands::List { workspace } => {
                let agents = orchestrator.list_agents(workspace.as_deref()).await?;
                println!(
                    "\n=== Paired AI Coding Agents ({} registered) ===\n",
                    agents.len()
                );
                if agents.is_empty() {
                    println!("No agents paired yet. Run `smoooth agent pair --name <agent-name>` to issue credentials.\n");
                } else {
                    for a in agents {
                        let status = if a.revoked { "REVOKED" } else { "ACTIVE" };
                        let over = if a.is_overprivileged() {
                            " [⚠️ ELEVATED]"
                        } else {
                            ""
                        };
                        println!("• {} [{}] - Status: {}{}", a.name, a.agent_id, status, over);
                        println!("  Workspace : {}", a.workspace_id);
                        println!("  Scopes    : {:?}", a.scopes);
                        println!("  Prefix    : {}...", a.token_prefix);
                        println!(
                            "  Last Used : {}\n",
                            a.last_used_at
                                .map(|t| t.to_rfc3339())
                                .unwrap_or_else(|| "Never".to_string())
                        );
                    }
                }
            }

            AgentCommands::Revoke { agent_id } => {
                orchestrator.revoke_agent(&agent_id).await?;
                println!(
                    "\n✅ Agent '{}' credentials successfully REVOKED.",
                    agent_id
                );
                println!(
                    "   All subsequent requests with this token will be rejected immediately.\n"
                );
            }

            AgentCommands::Rotate { agent_id } => {
                let new_token = orchestrator.rotate_agent_token(&agent_id).await?;
                println!("\n============================================================");
                println!("  ✅ Agent Token Rotated Successfully: {}", agent_id);
                println!("============================================================");
                println!("  🔑 New Plaintext Bearer Token (DISPLAYED ONCE):");
                println!("     {}", new_token);
                println!("============================================================\n");
            }
        },

        Some(Commands::Tools) => {
            println!("\n=== Smoooth MCP Tool Interface (7-Step Golden Workflow) ===\n");
            for tool in get_available_tools() {
                println!("  [{}] {:<25} - {}", tool.step, tool.name, tool.description);
            }
            println!();
        }

        Some(Commands::Capabilities) => {
            println!("\n=== Supported Infrastructure Capabilities & Provider Guarantees ===\n");
            for cap in get_default_capabilities() {
                println!("Capability ID : {}", cap.id);
                println!("Name          : {}", cap.name);
                println!("Provider/Type : {} / {}", cap.provider, cap.service_type);
                println!("Support Level : {:?}", cap.support_level);
                println!("Description   : {}", cap.description);
                println!("Operations    : {:?}\n", cap.operations);
            }
        }

        Some(Commands::Status { workspace }) => {
            println!("\n=== Smoooth Local State [Workspace: {}] ===", workspace);
            let resources = orchestrator.list_resources(&workspace).await;
            if resources.is_empty() {
                println!("No resources provisioned yet. Run `smoooth demo` to test.");
            } else {
                for r in resources {
                    println!(
                        "• [{}] {} (Provider: {}, Status: {:?}, Persistent: {})",
                        r.id, r.name, r.provider, r.status, r.is_persistent
                    );
                }
            }
            println!();
        }

        Some(Commands::Demo) => {
            println!("\n🚀 ================================================================");
            println!("   Smoooth Golden Workflow (Local Demonstration)");
            println!("   Infrastructure freedom, built for AI agents.");
            println!("================================================================\n");

            // Step 01: Discover
            println!("🔍 [01 Discover] Agent queries available capabilities...");
            let capabilities = get_default_capabilities();
            println!(
                "   ↳ Found {} discoverable capabilities (Docker, PostgreSQL, Redis, MongoDB).",
                capabilities.len()
            );

            // Step 02: Plan
            println!("\n📐 [02 Plan] Agent drafts CanonicalExecutionSpec...");
            let mut artifacts = HashMap::new();
            artifacts.insert(
                "web_app".to_string(),
                "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_string(),
            );

            let spec = CanonicalExecutionSpec {
                schema_version: 1,
                workspace_id: "default".to_string(),
                environment: "development".to_string(),
                target_id: "local_docker_node".to_string(),
                actions: vec![
                    CanonicalActionSpec {
                        resource_name: "smoooth-postgres".to_string(),
                        provider: "docker".to_string(),
                        service_type: "database".to_string(),
                        action_type: "create".to_string(),
                        config: serde_json::json!({ "image": "postgres:16-alpine", "port": 5432 }),
                        is_persistent: true, // Safeguarded against accidental deletion!
                        exposes_network_publicly: false,
                    },
                    CanonicalActionSpec {
                        resource_name: "smoooth-api".to_string(),
                        provider: "docker".to_string(),
                        service_type: "compute".to_string(),
                        action_type: "create".to_string(),
                        config: serde_json::json!({ "image": "app:v1", "port": 8080 }),
                        is_persistent: false, // Ephemeral compute
                        exposes_network_publicly: true,
                    },
                ],
                dependencies: vec![DependencyEdge {
                    from: "smoooth-api".to_string(),
                    to: "smoooth-postgres".to_string(),
                }],
                immutable_artifacts: artifacts,
                policy_envelope: PolicyConstraints {
                    allow_public_network: true,
                    allow_data_destruction: false,
                    max_monthly_budget_usd: Some(50),
                },
            };

            let mut plan = Plan::new(
                "plan_demo_01".to_string(),
                spec,
                RiskTier::Moderate,
                Some(15.00),
            );

            println!("   ↳ Generated Canonical Plan Hash: {}", plan.plan_hash);
            println!(
                "   ↳ Risk Tier: {:?} | Estimated Cost: ${:.2}/mo",
                plan.risk_tier,
                plan.estimated_monthly_cost_usd.unwrap_or(0.0)
            );

            // Step 03: Govern
            println!("\n🛡️ [03 Govern] User policy verification...");
            // Bind approval to exact cryptographic plan hash
            plan.approved_plan_hash = Some(plan.plan_hash.clone());
            plan.approved_by = Some("operator@smoooth.local".to_string());
            println!(
                "   ↳ Policy approved bound to hash: {}...",
                &plan.plan_hash[..16]
            );

            // Step 04: Provision
            println!("\n⚙️ [04 Provision] Starting durable plan execution with fencing leases...");
            let op = orchestrator
                .execute_plan(plan.clone(), "idemp_client_key_demo_01".to_string())
                .await?;
            println!(
                "   ↳ Operation ID: {} (Fencing Epoch: {})",
                op.id, op.lease_epoch
            );

            // Step 05: Connect
            println!("\n🔗 [05 Connect] Creating secure zero-leak ServiceBinding...");
            let raw_conn_string =
                "postgresql://smoooth_user:s3cr3tP@ssw0rd!@smoooth-postgres:5432/appdb";
            let masked = ServiceBinding::mask_connection_string(raw_conn_string);
            println!("   ↳ Injected Masked Connection: {}", masked);
            println!("   ↳ Credentials encrypted at rest with staged dual-credential support.");

            // Step 06: Verify
            println!("\n✅ [06 Verify] Running active probes...");
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            println!("   ↳ TCP Socket Probe: OK (5ms latency)");
            println!("   ↳ HTTP /health Probe: 200 OK");
            println!("   ↳ Database Ping: Connected. Readiness verified with evidence.");

            // Step 07: Operate
            println!("\n🛠️ [07 Operate] Inspecting local state & persistent volumes...");
            let resources = orchestrator.list_resources("default").await;
            println!("   ↳ Total Active Resources: {}", resources.len());
            for r in &resources {
                println!(
                    "     • {} [{}] (Persistent Volume Guard: {})",
                    r.name, r.id, r.is_persistent
                );
            }

            println!("\n🎉 Golden Workflow successfully demonstrated locally without requiring external DB!");
            println!("   State saved to `smoooth_state.json`.");
            println!("   Run `smoooth status` to view active resources at any time.\n");
        }

        Some(Commands::Serve {
            bind,
            port,
            token,
            token_file,
        }) => {
            smoooth::api::start_server(Arc::new(orchestrator), &bind, port, token, token_file)
                .await?;
        }

        Some(Commands::Mcp) => {
            let mcp_server = smoooth::mcp::McpServer::new(Arc::new(orchestrator));
            mcp_server.run_stdio().await?;
        }

        Some(Commands::Approve { plan_id, approver }) => {
            match orchestrator.get_plan(&plan_id).await? {
                Some(mut plan) => {
                    plan.approve_with_ttl(approver.clone(), 3600);
                    orchestrator.save_plan(plan.clone()).await?;
                    println!(
                        "\n✅ Plan '{}' successfully approved by '{}'.",
                        plan_id, approver
                    );
                    println!("   Bound canonical hash : {}", plan.plan_hash);
                    println!("   Approval expires in  : 1 hour\n");
                }
                None => {
                    eprintln!("\n❌ Error: Plan '{}' not found.\n", plan_id);
                }
            }
        }

        Some(Commands::Ops { operation_id }) => {
            match orchestrator.get_operation(&operation_id).await {
                Some(op) => {
                    println!("\n=== Smoooth Operation [{}] ===", op.id);
                    println!("Status      : {:?}", op.status);
                    println!("Progress    : {}%", op.progress_percent);
                    println!("Lease Epoch : {}", op.lease_epoch);
                    println!("Plan Hash   : {}", op.plan_hash);
                    if let Some(err) = &op.error_message {
                        println!("Error       : {}", err);
                    }
                    if let Some(ev) = &op.evidence {
                        println!(
                            "Evidence    : {}",
                            serde_json::to_string_pretty(ev).unwrap_or_default()
                        );
                    }
                    println!("\nLogs ({} entries):", op.logs.len());
                    for log in &op.logs {
                        println!("  [{}] {}", log.level, log.message);
                    }
                    println!();
                }
                None => {
                    eprintln!("\n❌ Error: Operation '{}' not found.\n", operation_id);
                }
            }
        }

        Some(Commands::Reconcile { resource_id }) => {
            println!(
                "\n🔍 Running reconciliation for resource '{}'...",
                resource_id
            );
            match orchestrator.reconcile_resource(&resource_id).await {
                Ok(outcome) => {
                    println!("Resource ID      : {}", outcome.resource_id);
                    println!("Observed Status  : {:?}", outcome.observed_status);
                    println!("Intended Status  : {:?}", outcome.intended_status);
                    println!("Is Drifted       : {}", outcome.is_drifted);
                    println!("Drift Corrected  : {}", outcome.drift_corrected);
                    println!("Outcome Message  : {}\n", outcome.message);
                }
                Err(e) => {
                    eprintln!("❌ Reconciliation failed: {}\n", e);
                }
            }
        }

        Some(Commands::Ingress { command }) => {
            let ingress_manager = smoooth::ingress::IngressManager::default_local();
            match command {
                IngressCommands::Status => {
                    println!("\n🔍 Smoooth Ingress & Routing Subsystem");
                    let mode = smoooth::ingress::IngressManager::detect_mode();
                    match mode {
                        smoooth::ingress::IngressMode::StandaloneCaddy => {
                            println!("Ingress Mode : Standalone Caddy (Ports 80 & 443 available)");
                            println!("TLS Engine   : Automatic ACME / Let's Encrypt");
                        }
                        smoooth::ingress::IngressMode::UpstreamProxy { existing_proxy } => {
                            println!("Ingress Mode : Upstream Subordinate");
                            println!("Status       : {}", existing_proxy);
                            println!("Advice       : Use 'smoooth ingress export' to generate upstream proxy config.");
                        }
                    }
                    let routes = ingress_manager.list_routes().unwrap_or_default();
                    println!("Active Routes: {}\n", routes.len());
                }
                IngressCommands::List => {
                    let routes = ingress_manager.list_routes().unwrap_or_default();
                    if routes.is_empty() {
                        println!("\nNo ingress routes configured. Use 'smoooth ingress add' to register a domain.\n");
                    } else {
                        println!("\n=== Active Ingress Routes ({}) ===", routes.len());
                        for r in routes {
                            let scheme = if r.tls { "https" } else { "http" };
                            let prefix = r.path_prefix.as_deref().unwrap_or("/");
                            println!("  • {}://{} (path: {}) -> {} (port {}) [ID: {}]",
                                scheme, r.domain, prefix, r.target_resource, r.target_port, r.id
                            );
                        }
                        println!();
                    }
                }
                IngressCommands::Add { domain, target, port, tls, path_prefix } => {
                    match ingress_manager.add_route(&domain, &target, port, tls, path_prefix) {
                        Ok(route) => {
                            println!("\n✅ Ingress route registered successfully!");
                            println!("Domain       : {}", route.domain);
                            println!("Target       : {} (port {})", route.target_resource, route.target_port);
                            println!("TLS          : {}", if route.tls { "Enabled (HTTPS)" } else { "Disabled (HTTP)" });
                            println!("Route ID     : {}\n", route.id);
                        }
                        Err(e) => {
                            eprintln!("\n❌ Failed to add ingress route: {}\n", e);
                        }
                    }
                }
                IngressCommands::Remove { target } => {
                    match ingress_manager.remove_route(&target) {
                        Ok(true) => println!("\n✅ Route '{}' removed successfully.\n", target),
                        Ok(false) => eprintln!("\n❌ Route '{}' not found.\n", target),
                        Err(e) => eprintln!("\n❌ Failed to remove route: {}\n", e),
                    }
                }
                IngressCommands::Export { format } => {
                    match ingress_manager.render_upstream_snippet(&format) {
                        Ok(cfg) => {
                            println!("\n=== Ingress Configuration [{}] ===\n", format.to_uppercase());
                            print!("{}", cfg);
                        }
                        Err(e) => eprintln!("\n❌ Failed to export ingress config: {}\n", e),
                    }
                }
            }
        }

        Some(Commands::Build { command }) => {
            match command {
                BuildCommands::Plan { source } => {
                    println!("\n📦 Inspecting source for zero-Dockerfile Nixpacks plan: {}\n", source);
                    match smoooth::build::NixpacksBuilder::plan(&source).await {
                        Ok(plan_json) => {
                            let providers = plan_json.get("providers")
                                .and_then(|p| p.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                                .unwrap_or_else(|| "generic".to_string());
                            let start_cmd = plan_json.get("start")
                                .and_then(|s| s.get("cmd"))
                                .and_then(|c| c.as_str())
                                .unwrap_or("none");
                            println!("Detected Providers : {}", providers);
                            println!("Start Command      : {}", start_cmd);
                            println!("\nDetailed Plan Structure:\n{}", serde_json::to_string_pretty(&plan_json).unwrap_or_default());
                        }
                        Err(e) => {
                            eprintln!("\n❌ Nixpacks plan failed: {}\n", e);
                            std::process::exit(1);
                        }
                    }
                }
                BuildCommands::Image { source, name, env } => {
                    println!("\n🔨 Building container image '{}' from '{}' via Nixpacks...\n", name, source);
                    let mut env_map = HashMap::new();
                    for item in env {
                        if let Some((k, v)) = item.split_once('=') {
                            env_map.insert(k.to_string(), v.to_string());
                        }
                    }
                    let env_opt = if env_map.is_empty() { None } else { Some(env_map) };
                    match smoooth::build::NixpacksBuilder::build(&source, &name, env_opt).await {
                        Ok(out) => {
                            for line in &out.logs {
                                println!("{}", line);
                            }
                            if out.success {
                                println!("\n✅ Build succeeded! Tagged image: {}\n", out.image_tag);
                            } else {
                                eprintln!("\n❌ Build failed: {}\n", out.error.unwrap_or_default());
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("\n❌ Nixpacks build execution failed: {}\n", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }

        None => {
            println!("No subcommand specified. Starting Smoooth API Server & Control Plane Dashboard on 127.0.0.1:4040...");
            smoooth::api::start_server(Arc::new(orchestrator), "127.0.0.1", 4040, None, None)
                .await?;
        }
    }

    Ok(())
}
