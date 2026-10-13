-- 0001_initial_control_plane.sql
-- Smoooth Control Plane Database Schema (PostgreSQL)

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 1. Workspaces
CREATE TABLE IF NOT EXISTS workspaces (
    id VARCHAR(64) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    max_monthly_budget_usd NUMERIC(10, 2) DEFAULT 100.00,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. Capabilities & Provider Guarantees
CREATE TABLE IF NOT EXISTS capabilities (
    id VARCHAR(64) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    provider VARCHAR(64) NOT NULL,
    service_type VARCHAR(64) NOT NULL,
    support_level VARCHAR(32) NOT NULL, -- 'managed', 'integrated', 'self_hosted', 'experimental'
    supports_persistent_volumes BOOLEAN NOT NULL DEFAULT FALSE,
    supports_staged_dual_credentials BOOLEAN NOT NULL DEFAULT FALSE,
    supports_safe_rollback BOOLEAN NOT NULL DEFAULT FALSE,
    schema JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. Execution Plans (Immutable & Hash-Bound)
CREATE TABLE IF NOT EXISTS plans (
    id VARCHAR(64) PRIMARY KEY,
    workspace_id VARCHAR(64) NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    schema_version INT NOT NULL DEFAULT 1,
    environment VARCHAR(32) NOT NULL DEFAULT 'development',
    plan_hash VARCHAR(64) NOT NULL UNIQUE,
    canonical_spec JSONB NOT NULL,
    risk_tier VARCHAR(32) NOT NULL, -- 'low', 'moderate', 'high', 'critical'
    estimated_monthly_cost_usd NUMERIC(10, 2),
    approved_plan_hash VARCHAR(64),
    approved_by VARCHAR(255),
    approved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 4. Idempotency Records (Composite Key: workspace_id + operation_type + client_key)
CREATE TABLE IF NOT EXISTS idempotency_records (
    workspace_id VARCHAR(64) NOT NULL,
    operation_type VARCHAR(64) NOT NULL,
    client_key VARCHAR(128) NOT NULL,
    request_hash VARCHAR(64) NOT NULL,
    operation_id VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (workspace_id, operation_type, client_key)
);

-- 5. Durable Operations (Leases with Monotonic Fencing Epochs)
CREATE TABLE IF NOT EXISTS operations (
    id VARCHAR(64) PRIMARY KEY,
    workspace_id VARCHAR(64) NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    operation_type VARCHAR(64) NOT NULL,
    plan_id VARCHAR(64) REFERENCES plans(id),
    plan_hash VARCHAR(64) NOT NULL,
    request_hash VARCHAR(64) NOT NULL,
    idempotency_client_key VARCHAR(128) NOT NULL,
    status VARCHAR(32) NOT NULL, -- 'pending', 'running', 'succeeded', 'failed', 'cancelled'
    progress_percent SMALLINT NOT NULL DEFAULT 0,
    lease_epoch BIGINT NOT NULL DEFAULT 1,
    lease_owner VARCHAR(128),
    lease_expires_at TIMESTAMPTZ,
    intended_state JSONB NOT NULL DEFAULT '{}'::jsonb,
    observed_state JSONB NOT NULL DEFAULT '{}'::jsonb,
    evidence JSONB,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 6. Resources & Persistent Storage Safeguards
CREATE TABLE IF NOT EXISTS resources (
    id VARCHAR(64) PRIMARY KEY,
    workspace_id VARCHAR(64) NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    service_type VARCHAR(64) NOT NULL,
    provider VARCHAR(64) NOT NULL,
    is_persistent BOOLEAN NOT NULL DEFAULT FALSE,
    status VARCHAR(32) NOT NULL, -- 'provisioning', 'active', 'degraded', 'stopped', 'terminated'
    configuration JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 7. Service Bindings & Staged Secret Rotation
CREATE TABLE IF NOT EXISTS service_bindings (
    id VARCHAR(64) PRIMARY KEY,
    workspace_id VARCHAR(64) NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    source_resource_id VARCHAR(64) NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    target_resource_id VARCHAR(64) NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    binding_type VARCHAR(64) NOT NULL,
    rotation_stage VARCHAR(32) NOT NULL DEFAULT 'active', -- 'active', 'overlapping', 'verifying', 'retired', 'rolled_back'
    encrypted_credentials BYTEA NOT NULL,
    dek_key_version INT NOT NULL DEFAULT 1,
    nonce BYTEA NOT NULL,
    exposed_env_keys JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 8. Immutable Audit Trail
CREATE TABLE IF NOT EXISTS audit_events (
    id BIGSERIAL PRIMARY KEY,
    workspace_id VARCHAR(64) NOT NULL,
    actor VARCHAR(255) NOT NULL,
    action VARCHAR(128) NOT NULL,
    target_resource_id VARCHAR(64),
    plan_hash VARCHAR(64),
    outcome VARCHAR(32) NOT NULL, -- 'success', 'denied', 'failed'
    redacted_evidence JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance & quick lease lookup
CREATE INDEX IF NOT EXISTS idx_operations_status_lease ON operations(status, lease_expires_at);
CREATE INDEX IF NOT EXISTS idx_resources_workspace ON resources(workspace_id);
CREATE INDEX IF NOT EXISTS idx_audit_events_workspace ON audit_events(workspace_id, created_at);
