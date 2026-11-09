-- Append-only audit journal. Audit writes must share the business transaction.
-- before_state/after_state must be allowlisted and redacted by the trusted service.
BEGIN;

CREATE SCHEMA audit;

CREATE TABLE audit.events (
    event_seq       BIGINT GENERATED ALWAYS AS IDENTITY UNIQUE,
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    actor_account_id UUID REFERENCES iam.accounts(id) ON DELETE RESTRICT,
    actor_kind      TEXT NOT NULL CHECK (actor_kind IN ('user', 'service', 'system')),
    action          TEXT NOT NULL CHECK (action = btrim(action) AND length(action) BETWEEN 1 AND 128),
    entity_schema   TEXT NOT NULL CHECK (entity_schema = btrim(entity_schema) AND length(entity_schema) BETWEEN 1 AND 63),
    entity_table    TEXT NOT NULL CHECK (entity_table = btrim(entity_table) AND length(entity_table) BETWEEN 1 AND 63),
    entity_id       UUID,
    outcome         TEXT NOT NULL CHECK (outcome IN ('succeeded', 'rejected', 'failed')),
    reason_code     TEXT CHECK (reason_code IS NULL OR length(reason_code) <= 128),
    request_id      UUID,
    correlation_id  UUID,
    before_state    JSONB,
    after_state     JSONB,
    metadata        JSONB NOT NULL DEFAULT '{}'::jsonb CHECK (jsonb_typeof(metadata) = 'object'),
    CONSTRAINT audit_actor_kind_consistency CHECK (
        (actor_kind = 'user' AND actor_account_id IS NOT NULL)
        OR (actor_kind IN ('service', 'system') AND actor_account_id IS NULL)
    ),
    CONSTRAINT audit_state_object CHECK (
        (before_state IS NULL OR jsonb_typeof(before_state) = 'object')
        AND (after_state IS NULL OR jsonb_typeof(after_state) = 'object')
    )
);
CREATE INDEX audit_events_time_idx ON audit.events (occurred_at DESC, event_seq DESC);
CREATE INDEX audit_events_actor_idx ON audit.events (actor_account_id, occurred_at DESC)
    WHERE actor_account_id IS NOT NULL;
CREATE INDEX audit_events_entity_idx ON audit.events (entity_schema, entity_table, entity_id, occurred_at DESC);
CREATE INDEX audit_events_request_idx ON audit.events (request_id) WHERE request_id IS NOT NULL;

CREATE FUNCTION audit.reject_event_mutation()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    RAISE EXCEPTION 'audit.events is append-only; % is forbidden', TG_OP
        USING ERRCODE = '55000';
END;
$$;
CREATE TRIGGER audit_events_no_update_delete
    BEFORE UPDATE OR DELETE ON audit.events
    FOR EACH ROW EXECUTE FUNCTION audit.reject_event_mutation();
CREATE TRIGGER audit_events_no_truncate
    BEFORE TRUNCATE ON audit.events
    FOR EACH STATEMENT EXECUTE FUNCTION audit.reject_event_mutation();

COMMENT ON SCHEMA audit IS 'Security and workforce audit records; deploy with a separate append-only writer role and external retention controls.';
COMMENT ON TABLE audit.events IS 'Append-only event journal. Store only allowlisted, redacted before/after data; row triggers are not protection against superusers.';

COMMIT;
