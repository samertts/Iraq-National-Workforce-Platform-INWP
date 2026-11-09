-- Destructive rollback: removes API idempotency receipts.
BEGIN;
DROP TABLE iam.idempotency_records;
COMMIT;
