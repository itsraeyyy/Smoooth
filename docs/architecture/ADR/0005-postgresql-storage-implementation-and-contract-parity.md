# ADR 0005: PostgreSQL Storage Implementation & Behavioral Contract Parity

## Status
Accepted

## Context
In Phase 1 Stage 1, the control plane established the unified `Storage` trait, local state snapshot synchronization, and fail-closed security invariants. In Stage 2, the relational schema and embedded migration runner were created, enforcing workspace boundaries and relational constraints at the database level.

Stage 3 requires a production-grade PostgreSQL backend (`PostgresStorage`) implementing all 36 methods of the `Storage` trait, delivering exact behavioral equivalence with `LocalStorage` without modifying orchestrator business logic or introducing external leak-prone pool dependencies.

## Decision

### 1. Zero-Leak Bounded Connection Management (`PostgresPool`)
Rather than pulling in heavy external connection pool crates, `PostgresPool` is implemented using native Tokio concurrency primitives:
- **Concurrency bounding**: `tokio::sync::Semaphore` enforces an upper bound (default 16 connections).
- **Idle connection reuse**: An internal `tokio::sync::Mutex<Vec<tokio_postgres::Client>>` manages idle clients.
- **Leak-free RAII recovery**: `PooledClient` wraps the client and an `OwnedSemaphorePermit`. On `Drop`, if the connection is healthy (`!client.is_closed()`), it returns the socket to the idle queue; if closed, it drops the connection, allowing subsequent acquisitions to establish fresh sockets.
- **Hermetic multi-schema scoping**: When configured with an optional schema, every connection automatically sets `SET search_path TO "<schema>", public;` on checkout, allowing hermetic parallel test execution.

### 2. Transactional and Relational Parity Across All 36 Storage Trait Methods
- **Workspace Isolation**: Database foreign keys enforce workspace tenancy. `PostgresStorage` ensures workspace records exist while preventing cross-workspace entity references.
- **Canonical Resource Identity**: On resource upsert, `(workspace_id, name)` uniqueness is checked first. When an adapter submits a resource with an updated configuration, the database retains the existing canonical ID (`res_{workspace_id}_{name}`) and original `created_at` timestamp.
- **Fail-Closed Cross-Workspace Service Bindings**: Service bindings enforce `source_resource_id <> target_resource_id` and require both resources to belong to the identical workspace. Cross-workspace binding attempts fail closed with `SmooothError::Security`.
- **Atomic Operation Reservation & Idempotency**: In a single database transaction, `SELECT ... FOR UPDATE` locks the idempotency record. If an entry exists with a mismatched request hash, `SmooothError::IdempotencyConflict` is returned. If matching, the existing operation is returned. If fresh, both the operation and idempotency record are inserted atomically.
- **Atomic Nonce Consumption**: Nonce replay protection is executed in a single atomic statement:
  ```sql
  INSERT INTO used_nonces (nonce, expires_at, consumed_at)
  VALUES ($1, $2, NOW())
  ON CONFLICT (nonce) DO NOTHING;
  ```
  Returns `true` if rows affected equals 1, and `false` if the nonce was expired or previously consumed.
- **Monotonic Fencing Generation Tokens**: Monotonically increments workspace fencing tokens in a single statement:
  ```sql
  INSERT INTO fencing_epochs (workspace_id, epoch, updated_at)
  VALUES ($1, 2, NOW())
  ON CONFLICT (workspace_id)
  DO UPDATE SET epoch = fencing_epochs.epoch + 1, updated_at = NOW()
  RETURNING epoch;
  ```
- **Batch Service Binding Rotation**: Checks current fingerprints of all target bindings with `FOR UPDATE`. If any existing fingerprint mismatches `expected_old_fingerprint`, or the new binding does not match `new_fingerprint`, the transaction rolls back immediately and fails closed.
- **Snapshot Export & Import Interoperability**: `export_snapshot` extracts a full or workspace-scoped `StateSnapshot`. `import_snapshot` restores entities transactionally into either `LocalStorage` or `PostgresStorage`.

### 3. Shared Behavioral Contract Test Suite
A comprehensive test harness (`tests/storage_contract_test.rs`) validates both backends against 9 identical behavioral invariants:
1. Workspace isolation across agents, resources, operations, and audit events.
2. Canonical resource identity preservation on `(workspace_id, name)` collisions.
3. Fail-closed cross-workspace service binding rejection.
4. Atomic operation reservation and idempotency conflict rejection.
5. Atomic nonce consumption and replay prevention.
6. Monotonic fencing generation token progression.
7. Batch binding rotation with transactional fingerprint verification.
8. Capability revocation registry checks.
9. Cross-backend snapshot round-trips (`LocalStorage` -> `PostgresStorage` and `PostgresStorage` -> `LocalStorage`).

## Consequences
- The control plane now operates interchangeably with either `LocalStorage` (file/memory) or `PostgresStorage` (PostgreSQL) backend.
- Zero changes to business logic or orchestrator contracts were required.
- Full parity is continuously verified through the shared 19-test contract suite.
