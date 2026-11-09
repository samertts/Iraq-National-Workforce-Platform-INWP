-- API idempotency receipts. Store only request digest and minimal replay response.
BEGIN;
CREATE TABLE iam.idempotency_records (
    account_id       UUID NOT NULL REFERENCES iam.accounts(id) ON DELETE RESTRICT,
    idempotency_key  UUID NOT NULL,
    request_digest   BYTEA NOT NULL CHECK (octet_length(request_digest) = 32),
    response_status  SMALLINT,
    response_body    JSONB,
    resource_id      UUID,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at     TIMESTAMPTZ,
    PRIMARY KEY (account_id, idempotency_key),
    CONSTRAINT idempotency_completion_consistency CHECK (
        (completed_at IS NULL AND response_status IS NULL AND response_body IS NULL)
        OR (completed_at IS NOT NULL AND response_status BETWEEN 200 AND 299 AND response_body IS NOT NULL)
    ),
    CONSTRAINT idempotency_response_object CHECK (response_body IS NULL OR jsonb_typeof(response_body) = 'object')
);
CREATE INDEX idempotency_records_created_idx ON iam.idempotency_records (created_at);
COMMENT ON TABLE iam.idempotency_records IS 'Per-account request receipts; stores a SHA-256 digest and minimal replay-safe response, not submitted PII.';
COMMIT;
