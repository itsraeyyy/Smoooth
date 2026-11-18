# ADR 0001: Strict Typed Execution Boundary & Prohibition of Raw Runtime Passthrough

## Status
Accepted

## Context
In the initial v0.1.1 implementation, workloads and containers were defined using open-ended `serde_json::Value` configuration objects, and several endpoints invoked raw shell commands (e.g. `docker restart`, `docker rm -f`, `docker run -v /var/run/docker.sock...`). 

Because Docker socket access on a Linux host confers effective root-level authority, allowing an AI agent or API caller to supply arbitrary runtime parameters creates an immediate privilege escalation and container breakout vulnerability. Specifically, an untrusted caller could supply privileged flags, arbitrary host mounts (`/:/host`), or host namespace options (`hostpid`, `hostnet`).

## Decision
We enforce a strict, strongly-typed `WorkloadSpec` execution boundary:
1. All infrastructure mutations must deserialize into explicit, validated Rust structs prior to reaching the execution broker.
2. Any configuration field not declared in `WorkloadSpec` is rejected with `400 Bad Request` / `SmooothError::Validation`.
3. The execution broker independently constructs container parameters (using `bollard`) and never passes raw caller-provided CLI arguments or arbitrary API flags.
4. Direct host process spawns (`tokio::process::Command::new("docker")`) are strictly forbidden across the codebase; all container operations must route through the `RuntimeProvider` interface.

## Consequences
### Positive
- Completely eliminates host filesystem mount escapes, arbitrary privileged flags, and namespace hijacking by AI agents.
- Provides predictable, deterministic validation errors to AI agents.
- Normalizes workload specifications so they can be executed identically across Docker, Podman, or future container runtimes.

### Negative / Trade-offs
- Advanced, non-standard Docker features cannot be used unless explicitly modeled in `WorkloadSpec`.
- Requires refactoring existing untyped migration previews to map into typed specifications.
