-- 0001_initial_control_plane.sql
-- Smoooth Control Plane Database Schema (PostgreSQL)
-- Phase 1 Task 1.2 — PostgreSQL Control-Plane Storage Foundation

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 1. Workspaces (Authoritative Tenant Boundary)
CREATE TABLE IF NOT EXISTS workspaces (
    id VARCHAR(64) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    max_monthly_budget_usd NUMERIC(10, 2) DEFAULT 100.00,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. Agents & Workload Access Identities
CREATE TABLE IF NOT EXISTS agents (
    agent_id VARCHAR(64) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    token_hash VARCHAR(128) NOT NULL UNIQUE,
    token_prefix VARCHAR(32) NOT NULL,
    scopes JSONB NOT NULL DEFAULT '[]'::jsonb,
    workspace_id VARCHAR(64) NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ
);

-- 3. Execution Plans (Immutable & Hash-Bound)
CREATE TABLE IF NOT EXISTS plans (
    id VARCHAR(64) PRIMARY KEY,
    workspace_id VARCHAR(64) NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    schema_version INT NOT NULL DEFAULT 1,
    environment VARCHAR(32) NOT NULL DEFAULT 'development',
    plan_hash VARCHAR(64) NOT NULL UNIQUE,
    canonical_spec JSONB NOT NULL,
    risk_tier VARCHAR(32) NOT NULL CHECK (risk_tier IN ('low', 'moderate', 'high', 'critical')),
    estimated_monthly_cost_usd NUMERIC(10, 2),
    approved_plan_hash VARCHAR(64),
    approved_by VARCHAR(255),
    approval JSONB,
    created_by VARCHAR(255) NOT NULL DEFAULT 'operator',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_plans_workspace_id UNIQUE (workspace_id, id)
);

-- 4. Durable Operations (Leases with Monotonic Fencing Epochs)
CREATE TABLE IF NOT EXISTS operations (
    id VARCHAR(64) PRIMARY KEY,
    workspace_id VARCHAR(64) NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    operation_type VARCHAR(64) NOT NULL,
    plan_id VARCHAR(64) NOT NULL,
    plan_hash VARCHAR(64) NOT NULL,
    request_hash VARCHAR(64) NOT NULL,
    idempotency_client_key VARCHAR(128) NOT NULL,
    status VARCHAR(32) NOT NULL CHECK (status IN ('pending', 'running', 'succeeded', 'failed', 'cancelled')),
    progress_percent SMALLINT NOT NULL DEFAULT 0 CHECK (progress_percent BETWEEN 0 AND 100),
    lease_epoch BIGINT NOT NULL DEFAULT 1,
    lease_owner VARCHAR(128),
    lease_expires_at TIMESTAMPTZ,
    intended_state JSONB NOT NULL DEFAULT '{}'::jsonb,
    observed_state JSONB NOT NULL DEFAULT '{}'::jsonb,
    logs JSONB NOT NULL DEFAULT '[]'::jsonb,
    evidence JSONB,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_operations_plan FOREIGN KEY (workspace_id, plan_id) REFERENCES plans(workspace_id, id) ON DELETE RESTRICT,
    CONSTRAINT uq_operations_workspace_id UNIQUE (workspace_id, id)
);

-- 5. Idempotency Records (Composite Key: workspace_id + operation_type + client_key)
CREATE TABLE IF NOT EXISTS idempotency_records (
    workspace_id VARCHAR(64) NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    operation_type VARCHAR(64) NOT NULL,
    client_key VARCHAR(128) NOT NULL,
    request_hash VARCHAR(64) NOT NULL,
    operation_id VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, operation_type, client_key),
    CONSTRAINT fk_idempotency_operation FOREIGN KEY (workspace_id, operation_id) REFERENCES operations(workspace_id, id) ON DELETE CASCADE
);

-- 6. Resources & Persistent Storage Safeguards (Canonical Identity: res_{workspace_id}_{name})
CREATE TABLE IF NOT EXISTS resources (
    id VARCHAR(64) PRIMARY KEY,
    workspace_id VARCHAR(64) NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    service_type VARCHAR(64) NOT NULL,
    provider VARCHAR(64) NOT NULL,
    is_persistent BOOLEAN NOT NULL DEFAULT FALSE,
    ownership VARCHAR(32) NOT NULL DEFAULT 'smoooth_managed' CHECK (ownership IN ('smoooth_managed', 'external', 'discovered')),
    status VARCHAR(32) NOT NULL DEFAULT 'provisioning' CHECK (status IN ('provisioning', 'active', 'degraded', 'stopping', 'stopped', 'terminated')),
    configuration JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_resources_workspace_name UNIQUE (workspace_id, name),
    CONSTRAINT uq_resources_workspace_id UNIQUE (workspace_id, id)
);

-- 7. Service Bindings & Staged Secret Rotation
CREATE TABLE IF NOT EXISTS service_bindings (
    id VARCHAR(64) PRIMARY KEY,
    workspace_id VARCHAR(64) NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    source_resource_id VARCHAR(64) NOT NULL,
    target_resource_id VARCHAR(64) NOT NULL,
    binding_type VARCHAR(64) NOT NULL,
    rotation_stage VARCHAR(32) NOT NULL DEFAULT 'active' CHECK (rotation_stage IN ('active', 'overlapping', 'verifying', 'retired', 'rolled_back')),
    ownership VARCHAR(32) NOT NULL DEFAULT 'smoooth_managed' CHECK (ownership IN ('smoooth_managed', 'external', 'discovered')),
    binding_version INT NOT NULL DEFAULT 1,
    key_fingerprint VARCHAR(128) NOT NULL DEFAULT '',
    resolved_endpoint JSONB,
    safe_connection_details JSONB,
    encrypted_credentials BYTEA NOT NULL,
    exposed_env_keys JSONB NOT NULL DEFAULT '[]'::jsonb,
    env_mappings JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_bindings_source_resource FOREIGN KEY (workspace_id, source_resource_id) REFERENCES resources(workspace_id, id) ON DELETE CASCADE,
    CONSTRAINT fk_bindings_target_resource FOREIGN KEY (workspace_id, target_resource_id) REFERENCES resources(workspace_id, id) ON DELETE CASCADE,
    CONSTRAINT chk_bindings_no_self_bind CHECK (source_resource_id <> target_resource_id),
    CONSTRAINT uq_bindings_source_target UNIQUE (workspace_id, source_resource_id, target_resource_id)
);

-- 8. Immutable Audit Trail
CREATE TABLE IF NOT EXISTS audit_events (
    id BIGSERIAL PRIMARY KEY,
    workspace_id VARCHAR(64) NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    actor VARCHAR(255) NOT NULL,
    action VARCHAR(128) NOT NULL,
    target_resource_id VARCHAR(64),
    plan_hash VARCHAR(64),
    outcome VARCHAR(32) NOT NULL CHECK (outcome IN ('success', 'denied', 'failed')),
    redacted_evidence JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 9. Capability Revocation Registry
CREATE TABLE IF NOT EXISTS capability_revocations (
    capability_id UUID PRIMARY KEY,
    reason TEXT NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

-- 10. Atomic Nonce Consumption Registry
CREATE TABLE IF NOT EXISTS used_nonces (
    nonce VARCHAR(128) PRIMARY KEY,
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 11. Monotonic Workspace Fencing Generation Tokens
CREATE TABLE IF NOT EXISTS fencing_epochs (
    workspace_id VARCHAR(64) PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    epoch BIGINT NOT NULL DEFAULT 1,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance & quick lease lookup
CREATE INDEX IF NOT EXISTS idx_operations_status_lease ON operations(status, lease_expires_at);
CREATE INDEX IF NOT EXISTS idx_operations_workspace ON operations(workspace_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_resources_workspace ON resources(workspace_id);
CREATE INDEX IF NOT EXISTS idx_resources_status ON resources(workspace_id, status);
CREATE INDEX IF NOT EXISTS idx_bindings_workspace ON service_bindings(workspace_id);
CREATE INDEX IF NOT EXISTS idx_bindings_target ON service_bindings(target_resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_events_workspace ON audit_events(workspace_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_agents_workspace ON agents(workspace_id);
CREATE INDEX IF NOT EXISTS idx_agents_token_hash ON agents(token_hash);
CREATE INDEX IF NOT EXISTS idx_used_nonces_expires_at ON used_nonces(expires_at);
CREATE INDEX IF NOT EXISTS idx_capability_revocations_expires_at ON capability_revocations(expires_at);
