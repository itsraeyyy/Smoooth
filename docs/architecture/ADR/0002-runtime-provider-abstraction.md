# ADR 0002: Runtime Provider Abstraction & Decoupling from Docker Engine

## Status
Accepted

## Context
Smoooth v0.1.1 couples core orchestration and database adapters directly to `DockerAdapter` and Docker-specific structs. Furthermore, endpoints in `api.rs` and `ingress` invoke the `docker` CLI directly via shell processes.

To realize the strategic blueprint—making infrastructure manageable without tying Smoooth permanently to Docker, and paving the way for rootless Podman or microVM backends (such as Firecracker)—the runtime execution layer must be decoupled behind a formal trait.

## Decision
1. Introduce a canonical, runtime-agnostic trait:
   ```rust
   #[async_trait]
   pub trait RuntimeProvider: Send + Sync {
       async fn create_and_start(&self, spec: &WorkloadSpec) -> Result<String>;
       async fn stop(&self, id: &str, timeout_secs: u32) -> Result<()>;
       async fn restart(&self, id: &str, timeout_secs: u32) -> Result<()>;
       async fn inspect(&self, id: &str) -> Result<ResourceStatus>;
       async fn logs(&self, id: &str, tail: usize) -> Result<Vec<String>>;
       async fn delete(&self, id: &str, force: bool) -> Result<()>;
       async fn health_check(&self, id: &str) -> Result<HealthEvidence>;
   }
   ```
2. Refactor `DockerAdapter` as the primary implementation of `RuntimeProvider`.
3. Eliminate all concrete Docker dependencies inside `PostgresAdapter`, `RedisAdapter`, and `MongoAdapter`. These adapters will now interact exclusively through `Arc<dyn RuntimeProvider>`.
4. Ensure any security policy or isolation feature that the underlying runtime cannot enforce results in an explicit error rather than a silent security downgrade.

## Consequences
### Positive
- Core orchestration and database management become completely runtime-independent.
- Podman, containerd, or sandbox runtimes can be added as drop-in implementations without modifying business or policy logic.
- Testing and mocking the runtime layer becomes trivial without requiring a live Docker daemon.

### Negative / Trade-offs
- Requires an adapter refactoring pass to convert existing `DockerAdapter` calls to `RuntimeProvider` methods.
- Dynamic feature discovery is needed so the control plane knows which optional capabilities a specific runtime supports.
