# Smoooth 🚀

> **Infrastructure freedom, built for AI agents.**  
> *Supabase-level convenience across the software stack, with the freedom to choose the technologies that fit the product.*

Based on the [Smoooth v2 MVP PRD](Smoooth_PRD_v2.md).

---

## 📁 Architecture Overview

```text
Smoooth/
├── Cargo.toml                              # Cargo manifest and dependencies
├── src/
│   ├── lib.rs                              # Library root
│   ├── main.rs                             # CLI entrypoint
│   ├── error.rs                            # Unified error types
│   ├── models/                             # Domain models
│   │   ├── capability.rs                   # Capabilities & support levels
│   │   ├── resource.rs                     # Infrastructure resources
│   │   ├── plan.rs                         # Execution plans & cryptographic hashes
│   │   ├── operation.rs                    # Durable operations & distributed leases
│   │   └── binding.rs                      # First-class service connections & secrets
│   ├── engine/                             # Core orchestration & lifecycle engine
│   │   └── orchestrator.rs                 # Plan execution pipeline & reconciliation
│   ├── adapters/                           # Extensible provider adapters
│   │   └── docker.rs                       # Docker runtime adapter
│   └── mcp/                                # Model Context Protocol (MCP) server
│       └── tools.rs                        # Complete workflow tool definitions
├── Smoooth_PRD_v2.md                       # Smoooth v2 MVP PRD (Official Canonical Spec)
```

---

## 🛠️ Prerequisites

To build and run this Rust project, Rust and Cargo must be installed on your system.

### Installing Rust via `rustup` on Windows:

1. **Option A (winget)**:
   ```powershell
   winget install Rustlang.Rustup
   ```

2. **Option B (Official Installer)**:
   Download and run `rustup-init.exe` from [rustup.rs](https://rustup.rs).

After installation, ensure `%USERPROFILE%\.cargo\bin` is added to your environment `PATH` and reopen your terminal.

---

## 🚀 Running the Project

Once Rust is installed:

```powershell
# Build the project
cargo build

# Run the CLI demo
cargo run

# List registered MCP tools
cargo run -- tools

# List supported infrastructure capabilities
cargo run -- capabilities
```
