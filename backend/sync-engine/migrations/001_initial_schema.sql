-- INWP Sync Engine — Initial Schema
-- PostgreSQL-only. Database: inwp_sync, Schema: sync

CREATE SCHEMA IF NOT EXISTS sync;

-- ============================================================================
-- Node Registry
-- ============================================================================

CREATE TABLE IF NOT EXISTS sync.node_registry (
    node_id             UUID PRIMARY KEY,
    node_type           TEXT NOT NULL CHECK (node_type IN ('national_hub', 'regional_relay', 'edge', 'mobile', 'dr_replica')),
    node_name           TEXT NOT NULL,
    ministry_id         UUID NOT NULL,
    site_id             UUID NOT NULL,
    region              TEXT NOT NULL,
    certificate_serial  TEXT NOT NULL DEFAULT '',
    public_key          BYTEA NOT NULL,
    address             TEXT NOT NULL DEFAULT '',
    port                INT NOT NULL DEFAULT 0,
    capabilities        JSONB NOT NULL DEFAULT '{}',
    status              TEXT NOT NULL DEFAULT 'offline' CHECK (status IN ('online', 'offline', 'suspected', 'recovering', 'quarantined')),
    last_heartbeat      TIMESTAMPTZ NOT NULL DEFAULT now(),
    first_seen          TIMESTAMPTZ NOT NULL DEFAULT now(),
    metadata            JSONB DEFAULT '{}'
);

CREATE INDEX idx_node_registry_region ON sync.node_registry(region);
CREATE INDEX idx_node_registry_status ON sync.node_registry(status);
CREATE INDEX idx_node_registry_type ON sync.node_registry(node_type);
CREATE INDEX idx_node_registry_ministry ON sync.node_registry(ministry_id);

-- ============================================================================
-- Sync Checkpoints (per partition, per node)
-- ============================================================================

CREATE TABLE IF NOT EXISTS sync.sync_checkpoint (
    node_id         UUID NOT NULL REFERENCES sync.node_registry(node_id) ON DELETE CASCADE,
    partition_key   TEXT NOT NULL,
    merkle_root     BYTEA NOT NULL DEFAULT '',
    last_sync_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    synced_events   BIGINT NOT NULL DEFAULT 0,
    last_error      TEXT,
    PRIMARY KEY (node_id, partition_key)
);

CREATE INDEX idx_sync_checkpoint_merkle ON sync.sync_checkpoint USING btree (merkle_root)
    WHERE merkle_root != '';

-- ============================================================================
-- Sync Batch Log (immutable receipts)
-- ============================================================================

CREATE TABLE IF NOT EXISTS sync.sync_batch_log (
    sync_id             UUID PRIMARY KEY,
    source_node         UUID NOT NULL REFERENCES sync.node_registry(node_id),
    target_node         UUID NOT NULL REFERENCES sync.node_registry(node_id),
    partition_key       TEXT NOT NULL,
    direction           TEXT NOT NULL CHECK (direction IN ('upload', 'download', 'bidirectional')),
    events_count        INT NOT NULL DEFAULT 0,
    bytes_transferred   BIGINT NOT NULL DEFAULT 0,
    conflict_count      INT NOT NULL DEFAULT 0,
    conflicts_auto      INT NOT NULL DEFAULT 0,
    conflicts_manual    INT NOT NULL DEFAULT 0,
    local_merkle        BYTEA NOT NULL,
    remote_merkle       BYTEA NOT NULL,
    source_sig          BYTEA NOT NULL,
    target_sig          BYTEA NOT NULL,
    compression_ratio   REAL NOT NULL DEFAULT 1.0,
    duration_ms         BIGINT NOT NULL DEFAULT 0,
    error               TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sync_batch_log_source ON sync.sync_batch_log(source_node);
CREATE INDEX idx_sync_batch_log_target ON sync.sync_batch_log(target_node);
CREATE INDEX idx_sync_batch_log_partition ON sync.sync_batch_log(partition_key);
CREATE INDEX idx_sync_batch_log_created ON sync.sync_batch_log(created_at DESC);

-- ============================================================================
-- Sync Queue (pending events for outbound sync)
-- ============================================================================

CREATE TABLE IF NOT EXISTS sync.sync_queue (
    queue_id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    node_id          UUID NOT NULL REFERENCES sync.node_registry(node_id) ON DELETE CASCADE,
    partition_key    TEXT NOT NULL,
    event_id         UUID NOT NULL,
    event_type       TEXT NOT NULL,
    payload          BYTEA NOT NULL,
    priority         INT NOT NULL DEFAULT 5 CHECK (priority BETWEEN 1 AND 10),
    status           TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    next_retry_at    TIMESTAMPTZ,
    retry_count      INT NOT NULL DEFAULT 0,
    last_error       TEXT
);

CREATE INDEX idx_sync_queue_priority ON sync.sync_queue(priority, created_at)
    WHERE status = 'pending';
CREATE INDEX idx_sync_queue_node_status ON sync.sync_queue(node_id, status);
CREATE INDEX idx_sync_queue_retry ON sync.sync_queue(next_retry_at)
    WHERE status = 'failed' AND next_retry_at IS NOT NULL;

-- ============================================================================
-- Sync Conflicts
-- ============================================================================

CREATE TABLE IF NOT EXISTS sync.sync_conflicts (
    conflict_id      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sync_id          UUID REFERENCES sync.sync_batch_log(sync_id),
    partition_key    TEXT NOT NULL,
    record_id        TEXT NOT NULL,
    record_type      TEXT NOT NULL,
    local_version    JSONB NOT NULL,
    remote_version   JSONB NOT NULL,
    strategy         TEXT NOT NULL DEFAULT 'lww',
    status           TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'resolved', 'escalated', 'quarantined', 'expired')),
    auto_resolvable  BOOLEAN NOT NULL DEFAULT false,
    escalated_at     TIMESTAMPTZ,
    resolved_by      UUID,
    resolution       TEXT,
    resolved_at      TIMESTAMPTZ,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_sync_conflicts_status ON sync.sync_conflicts(status);
CREATE INDEX idx_sync_conflicts_partition ON sync.sync_conflicts(partition_key);
CREATE INDEX idx_sync_conflicts_record ON sync.sync_conflicts(record_id);

-- ============================================================================
-- Event Store (append-only event log for replay)
-- ============================================================================

CREATE TABLE IF NOT EXISTS sync.event_store (
    event_id         UUID PRIMARY KEY,
    event_seq        BIGSERIAL NOT NULL,
    node_id          UUID NOT NULL,
    event_type       TEXT NOT NULL,
    partition_key    TEXT NOT NULL,
    payload          BYTEA NOT NULL,
    version_vector   JSONB NOT NULL,
    local_timestamp  BIGINT NOT NULL,
    signature        BYTEA NOT NULL DEFAULT '',
    signing_key_id   TEXT NOT NULL DEFAULT '',
    schema_version   TEXT NOT NULL DEFAULT '1.0',
    metadata         JSONB DEFAULT '{}',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_event_store_partition ON sync.event_store(partition_key, created_at);
CREATE INDEX idx_event_store_type ON sync.event_store(event_type);
CREATE INDEX idx_event_store_seq ON sync.event_store(event_seq);
CREATE UNIQUE INDEX idx_event_store_seq_unique ON sync.event_store(partition_key, event_seq);

-- ============================================================================
-- Dead Letter Queue
-- ============================================================================

CREATE TABLE IF NOT EXISTS sync.sync_dead_letter (
    dlq_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sync_id         UUID,
    event_id        UUID NOT NULL,
    partition_key   TEXT NOT NULL,
    payload         BYTEA NOT NULL,
    error_message   TEXT NOT NULL,
    error_code      TEXT NOT NULL,
    retry_count     INT NOT NULL DEFAULT 0,
    failed_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_retry_at   TIMESTAMPTZ,
    status          TEXT NOT NULL DEFAULT 'pending_review' CHECK (status IN ('pending_review', 'reviewed', 'resolved', 'discarded')),
    reviewed_by     UUID,
    resolution      TEXT,
    resolved_at     TIMESTAMPTZ
);

CREATE INDEX idx_sync_dead_letter_status ON sync.sync_dead_letter(status);

-- ============================================================================
-- Heartbeat Log
-- ============================================================================

CREATE TABLE IF NOT EXISTS sync.heartbeat_log (
    heartbeat_id    BIGSERIAL PRIMARY KEY,
    node_id         UUID NOT NULL REFERENCES sync.node_registry(node_id),
    status          TEXT NOT NULL,
    queue_depth     BIGINT NOT NULL DEFAULT 0,
    events_pending  BIGINT NOT NULL DEFAULT 0,
    cpu_load        REAL,
    memory_bytes    BIGINT,
    disk_bytes      BIGINT,
    partition_roots JSONB DEFAULT '{}',
    received_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_heartbeat_log_node ON sync.heartbeat_log(node_id, received_at DESC);

-- ============================================================================
-- Recovery State
-- ============================================================================

CREATE TABLE IF NOT EXISTS sync.recovery_state (
    node_id             UUID PRIMARY KEY REFERENCES sync.node_registry(node_id),
    phase               TEXT NOT NULL DEFAULT 'detection',
    autonomous_mode     BOOLEAN NOT NULL DEFAULT false,
    pending_events      BIGINT NOT NULL DEFAULT 0,
    synced_events       BIGINT NOT NULL DEFAULT 0,
    conflict_count      INT NOT NULL DEFAULT 0,
    last_reconnect      TIMESTAMPTZ,
    reconnect_attempts  INT NOT NULL DEFAULT 0,
    last_error          TEXT,
    started_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================================
-- Row-Level Security for Multi-Tenancy
-- ============================================================================

ALTER TABLE sync.node_registry ENABLE ROW LEVEL SECURITY;
ALTER TABLE sync.sync_checkpoint ENABLE ROW LEVEL SECURITY;
ALTER TABLE sync.sync_batch_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE sync.sync_queue ENABLE ROW LEVEL SECURITY;
ALTER TABLE sync.sync_conflicts ENABLE ROW LEVEL SECURITY;
ALTER TABLE sync.event_store ENABLE ROW LEVEL SECURITY;

-- Ministry isolation policies
CREATE POLICY ministry_isolation ON sync.node_registry
    USING (ministry_id = current_setting('app.current_ministry_id')::UUID);

CREATE POLICY ministry_isolation ON sync.sync_checkpoint
    USING (node_id IN (
        SELECT node_id FROM sync.node_registry
        WHERE ministry_id = current_setting('app.current_ministry_id')::UUID
    ));

CREATE POLICY ministry_isolation ON sync.sync_batch_log
    USING (source_node IN (
        SELECT node_id FROM sync.node_registry
        WHERE ministry_id = current_setting('app.current_ministry_id')::UUID
    ));

-- ============================================================================
-- Maintenance Functions
-- ============================================================================

CREATE OR REPLACE FUNCTION sync.purge_old_events(retention_days INT DEFAULT 365)
RETURNS BIGINT AS $$
DECLARE
    deleted BIGINT;
BEGIN
    DELETE FROM sync.event_store
    WHERE created_at < now() - (retention_days || ' days')::INTERVAL;
    GET DIAGNOSTICS deleted = ROW_COUNT;
    RETURN deleted;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION sync.purge_completed_queue(retention_hours INT DEFAULT 24)
RETURNS BIGINT AS $$
DECLARE
    deleted BIGINT;
BEGIN
    DELETE FROM sync.sync_queue
    WHERE status = 'completed'
      AND created_at < now() - (retention_hours || ' hours')::INTERVAL;
    GET DIAGNOSTICS deleted = ROW_COUNT;
    RETURN deleted;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION sync.update_heartbeat()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE sync.node_registry
    SET last_heartbeat = NEW.received_at, status = 'online'
    WHERE node_id = NEW.node_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER after_heartbeat_insert
    AFTER INSERT ON sync.heartbeat_log
    FOR EACH ROW
    EXECUTE FUNCTION sync.update_heartbeat();
