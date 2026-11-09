-- Destructive rollback: removes the organizational schemas and all their data.
BEGIN;
DROP SCHEMA org CASCADE;
COMMIT;
