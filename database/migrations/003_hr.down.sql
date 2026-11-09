-- Destructive rollback: removes HR tables/functions and all their data.
-- Org/IAM are retained; drop HR before IAM and org migrations.
BEGIN;
ALTER TABLE org.organizations DROP CONSTRAINT IF EXISTS organizations_created_by_fk;
ALTER TABLE org.organizations DROP CONSTRAINT IF EXISTS organizations_updated_by_fk;
ALTER TABLE org.units DROP CONSTRAINT IF EXISTS units_created_by_fk;
ALTER TABLE org.units DROP CONSTRAINT IF EXISTS units_updated_by_fk;
ALTER TABLE org.sites DROP CONSTRAINT IF EXISTS sites_created_by_fk;
ALTER TABLE org.sites DROP CONSTRAINT IF EXISTS sites_updated_by_fk;
DROP SCHEMA hr CASCADE;
COMMIT;
