-- Destructive rollback: removes IAM tables and all their data. Roll back dependent migrations first.
BEGIN;
DROP SCHEMA iam CASCADE;
COMMIT;
