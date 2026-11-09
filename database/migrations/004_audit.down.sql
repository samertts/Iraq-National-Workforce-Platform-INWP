-- Destructive rollback: removes the audit journal and all recorded events.
BEGIN;
DROP SCHEMA audit CASCADE;
COMMIT;
