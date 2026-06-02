CREATE SCHEMA IF NOT EXISTS attendance;
CREATE SCHEMA IF NOT EXISTS sync;

CREATE TYPE attendance.clock_event_type AS ENUM (
    'clock_in', 'clock_out', 'break_start', 'break_end', 'manual_correction'
);

CREATE TYPE attendance.sync_status AS ENUM (
    'local_only', 'synced', 'conflicted', 'resolved'
);

CREATE TYPE attendance.exception_type AS ENUM (
    'LATE_CLOCK_IN', 'EARLY_CLOCK_OUT', 'MISSING_CLOCK_IN',
    'MISSING_CLOCK_OUT', 'MISSING_BREAK', 'EXCEEDED_MAX_HOURS',
    'DUPLICATE_CLOCK', 'BIOMETRIC_MISMATCH', 'GPS_OUT_OF_RANGE'
);

CREATE TYPE attendance.exception_severity AS ENUM (
    'LOW', 'MEDIUM', 'HIGH', 'CRITICAL'
);

CREATE TYPE attendance.justification_type AS ENUM (
    'SICKNESS', 'OFFICIAL_DUTY', 'TECHNICAL_ISSUE', 'OTHER'
);

CREATE TABLE attendance.clock_events (
    id              UUID NOT NULL,
    employee_id     UUID NOT NULL,
    ministry_id     UUID NOT NULL,
    site_id         UUID NOT NULL,
    device_id       UUID NOT NULL,
    event_type      attendance.clock_event_type NOT NULL,
    event_time      TIMESTAMPTZ NOT NULL,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    biometric_match DOUBLE PRECISION,
    latitude        DOUBLE PRECISION,
    longitude       DOUBLE PRECISION,
    ip_address      INET,
    sync_id         UUID,
    source_node_id  UUID,
    sync_status     attendance.sync_status DEFAULT 'local_only',
    version         BIGINT DEFAULT 1,

    PRIMARY KEY (id, recorded_at)
) PARTITION BY RANGE (recorded_at);

CREATE TABLE attendance.clock_events_2026_q2 PARTITION OF attendance.clock_events
    FOR VALUES FROM ('2026-04-01') TO ('2026-07-01');

CREATE TABLE attendance.clock_events_2026_q3 PARTITION OF attendance.clock_events
    FOR VALUES FROM ('2026-07-01') TO ('2026-10-01');

CREATE TABLE attendance.clock_events_default PARTITION OF attendance.clock_events
    FOR VALUES FROM ('2026-10-01') TO ('2030-01-01');

CREATE INDEX idx_clock_events_employee_time
    ON attendance.clock_events (employee_id, event_time DESC);
CREATE INDEX idx_clock_events_site_time
    ON attendance.clock_events (site_id, event_time DESC);
CREATE INDEX idx_clock_events_device_time
    ON attendance.clock_events (device_id, event_time DESC);
CREATE INDEX idx_clock_events_sync_status
    ON attendance.clock_events (sync_status) WHERE sync_status = 'local_only';

ALTER TABLE attendance.clock_events ENABLE ROW LEVEL SECURITY;

CREATE POLICY ministry_isolation ON attendance.clock_events
    USING (ministry_id = current_setting('app.current_ministry_id')::UUID);

CREATE TABLE attendance.shifts (
    id              UUID PRIMARY KEY,
    ministry_id     UUID NOT NULL,
    site_id         UUID NOT NULL,
    name            TEXT NOT NULL,
    start_time      TIMESTAMPTZ NOT NULL,
    end_time        TIMESTAMPTZ NOT NULL,
    grace_period    BIGINT NOT NULL DEFAULT 900,
    break_duration  BIGINT NOT NULL DEFAULT 1800,
    overtime_policy JSONB,
    is_active       BOOLEAN NOT NULL DEFAULT true,
    version         BIGINT NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_shifts_site_active ON attendance.shifts (site_id) WHERE is_active = true;

ALTER TABLE attendance.shifts ENABLE ROW LEVEL SECURITY;
CREATE POLICY shifts_ministry_isolation ON attendance.shifts
    USING (ministry_id = current_setting('app.current_ministry_id')::UUID);

CREATE TABLE attendance.attendance_policies (
    id              UUID PRIMARY KEY,
    ministry_id     UUID NOT NULL,
    site_id         UUID,
    name            TEXT NOT NULL,
    rules           JSONB NOT NULL,
    effective_from  TIMESTAMPTZ NOT NULL,
    effective_to    TIMESTAMPTZ,
    version         INT NOT NULL DEFAULT 1,
    supersedes      UUID REFERENCES attendance.attendance_policies(id),
    approved_by     UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_policies_site_active
    ON attendance.attendance_policies (site_id) WHERE effective_to IS NULL;

ALTER TABLE attendance.attendance_policies ENABLE ROW LEVEL SECURITY;
CREATE POLICY policies_ministry_isolation ON attendance.attendance_policies
    USING (ministry_id = current_setting('app.current_ministry_id')::UUID);

CREATE TABLE attendance.attendance_exceptions (
    id                        UUID PRIMARY KEY,
    employee_id               UUID NOT NULL,
    clock_event_id            UUID,
    exception_type            attendance.exception_type NOT NULL,
    severity                  attendance.exception_severity NOT NULL,
    description               TEXT NOT NULL,
    occurred_at               TIMESTAMPTZ NOT NULL,
    detected_at               TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    justification_reason      TEXT,
    justification_type        attendance.justification_type,
    justification_submitted_at TIMESTAMPTZ,
    resolved_at               TIMESTAMPTZ,
    resolved_by               UUID,
    escalated_at              TIMESTAMPTZ
);

CREATE INDEX idx_exceptions_employee
    ON attendance.attendance_exceptions (employee_id, occurred_at DESC);
CREATE INDEX idx_exceptions_unresolved
    ON attendance.attendance_exceptions (detected_at) WHERE resolved_at IS NULL;

ALTER TABLE attendance.attendance_exceptions ENABLE ROW LEVEL SECURITY;
CREATE POLICY exceptions_ministry_isolation ON attendance.attendance_exceptions
    USING (true);

CREATE TABLE sync.outbox (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id       UUID NOT NULL,
    entity_type     TEXT NOT NULL,
    sync_id         UUID NOT NULL,
    source_node_id  UUID NOT NULL,
    status          TEXT NOT NULL DEFAULT 'pending',
    payload         JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_outbox_status ON sync.outbox (status) WHERE status = 'pending';

CREATE OR REPLACE FUNCTION attendance.notify_clock_event()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify('attendance_events', row_to_json(NEW)::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_clock_event_notify
    AFTER INSERT ON attendance.clock_events
    FOR EACH ROW
    EXECUTE FUNCTION attendance.notify_clock_event();
