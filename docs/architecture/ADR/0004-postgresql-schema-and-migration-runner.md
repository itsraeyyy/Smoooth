# ADR 0004: PostgreSQL Control-Plane Schema & Embedded Migration Runner

## Status
Accepted

## Context
As part of Phase 1 Task 1.2 (PostgreSQL Control-Plane Storage Foundation), Smoooth requires a durable, multi-tenant relational persistence tier to replace file-based `LocalStorage`. 

Stage 1 established strict architectural contracts in `LocalStorage`:
1. Authoritative workspace boundaries.
2. Canonical resource identities (`res_{workspace_id}_{name}`).
3. Fail-closed binding resolution and cross-workspace isolation.
4. Atomic idempotency reservation and operation pairing.
5. Atomic nonce replay protection.
6. Monotonic fencing generation tokens.

Stage 2 focuses exclusively on creating the hardened PostgreSQL schema, versioned SQL migrations, and an embedded transactional migration runner without introducing runtime storage changes (`PostgresStorage` is strictly deferred to Stage 3).

## Decision

### 1. Hardened Control-Plane Database Schema
We define an authoritative 11-table schema in `migrations/0001_initial_control_plane.sql`:

1. `workspaces`: Authoritative multi-tenant isolation root (`id` PRIMARY KEY, budget, timestamps).
2. `agents`: Workload access identities referencing `workspaces(id)` ON DELETE CASCADE, with unique `token_hash`.
3. `plans`: Immutable execution specs bound to `workspaces(id)` with `UNIQUE (workspace_id, id)`, unique `plan_hash`, and checked `risk_tier`.
4. `operations`: Distributed leases with monotonic fencing generation tokens (`lease_epoch`), progress tracking (`0..100`), status checks, and composite foreign key `(workspace_id, plan_id)` referencing `plans(workspace_id, id)`.
5. `idempotency_records`: Composite primary key `(workspace_id, operation_type, client_key)` enforcing exact idempotency reservation with composite foreign key `(workspace_id, operation_id)` referencing `operations(workspace_id, id)`. Eliminates orphan or cross-workspace idempotency references.
6. `resources`: Persisted workload resources with canonical identity `UNIQUE (workspace_id, name)` and `UNIQUE (workspace_id, id)`.
7. `service_bindings`: Relational bindings between resources enforcing cross-workspace isolation at the database engine level via composite foreign keys:
   - `FOREIGN KEY (workspace_id, source_resource_id) REFERENCES resources(workspace_id, id) ON DELETE CASCADE`
   - `FOREIGN KEY (workspace_id, target_resource_id) REFERENCES resources(workspace_id, id) ON DELETE CASCADE`
   - `CHECK (source_resource_id <> target_resource_id)` preventing loopback self-bindings.
   - `UNIQUE (workspace_id, source_resource_id, target_resource_id)` preventing duplicate bindings.
8. `audit_events`: Append-only immutable audit trail referencing `workspaces(id)`.
9. `capability_revocations`: Revocation registry indexed by expiration timestamp.
10. `used_nonces`: Replay-attack prevention table with primary key `nonce` and expiration index.
11. `fencing_epochs`: Monotonic generation token table per workspace.

### 2. Embedded Migration Runner Architecture
We implement `MigrationRunner` in `src/storage/postgres/migration.rs`:

1. **Distributed Concurrency Control**:
   Acquires a PostgreSQL session-level advisory lock (`pg_advisory_lock(0x534D4F4F4F5448)`) before inspecting or modifying the database schema. The lock is guaranteed to be released on completion or failure.
2. **Schema Ledger Tracking**:
   Initializes and maintains `schema_migrations` recording `version` (BIGINT PK), `name`, `checksum` (SHA-256), `applied_at` (TIMESTAMPTZ), and `execution_time_ms` (BIGINT).
3. **Fail-Closed Checksum Verification**:
   Before running any pending migration, previously applied migrations are compared against the codebase. Any checksum discrepancy triggers immediate `SmooothError::Storage` abort to prevent executing migrations on a tampered or corrupted database.
4. **Out-of-Order Detection**:
   Retroactive migrations with versions lower than the highest applied migration are rejected.
5. **Transactional Atomicity**:
   Each migration runs in an isolated database transaction (`BEGIN` -> DDL execution -> ledger update -> `COMMIT`). If any statement fails, the transaction is rolled back and no partial schema or ledger changes persist.

## Scope Boundaries (Stage 2)
The following boundaries are strictly enforced:
- **No `PostgresStorage`**: Stage 2 does not implement `impl Storage for PostgresStorage`.
- **No Runtime Replacement**: `LocalStorage` remains the active runtime storage backend for all commands and workflows.
- **No Speculative Enterprise Hierarchy**: No premature organizations or projects tables; `workspaces` is the sole tenant boundary.
- **No Master-Key Rotation Workflows**: Secret rotation and key migration logic are deferred to later stages.

## Consequences

### Positive
- Enforces multi-tenant isolation, resource uniqueness, and binding validity at the database engine level via composite foreign keys and check constraints.
- Prevents split-brain and concurrent migration races via PostgreSQL advisory locks.
- Guarantees deterministic database rollbacks on failed migrations.
- Fails closed on any unexpected schema alterations or tampering.

### Negative / Trade-offs
- Composite foreign keys require `UNIQUE (workspace_id, id)` constraints on parent tables.
- Running migrations requires temporary exclusive advisory lock ownership on the PostgreSQL instance.
