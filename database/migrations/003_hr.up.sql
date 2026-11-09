-- INWP workforce registry. No national identifier, password, or plaintext sensitive profile is stored.
BEGIN;

CREATE SCHEMA hr;

CREATE TABLE hr.employees (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    registry_org_id     UUID NOT NULL REFERENCES org.organizations(id) ON DELETE RESTRICT,
    employee_code       TEXT NOT NULL CHECK (employee_code = btrim(employee_code) AND length(employee_code) BETWEEN 1 AND 80),
    employment_status   TEXT NOT NULL DEFAULT 'pending' CHECK (
        employment_status IN ('pending', 'active', 'on_leave', 'suspended', 'separated')
    ),
    hired_on            DATE,
    separated_on        DATE,
    version             BIGINT NOT NULL DEFAULT 1 CHECK (version > 0),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by          UUID,
    updated_by          UUID,
    CONSTRAINT employees_registry_code_uq UNIQUE (registry_org_id, employee_code),
    CONSTRAINT employees_separation_date CHECK (separated_on IS NULL OR hired_on IS NULL OR separated_on >= hired_on),
    CONSTRAINT employees_id_org_key UNIQUE (id, registry_org_id)
);
CREATE INDEX employees_org_status_idx ON hr.employees (registry_org_id, employment_status);

-- Sensitive profile data is encrypted by the trusted application/KMS envelope-encryption flow.
-- Keep only ciphertext and a non-secret key reference here; do not place keys in PostgreSQL.
CREATE TABLE hr.employee_private_profiles (
    employee_id     UUID PRIMARY KEY REFERENCES hr.employees(id) ON DELETE RESTRICT,
    ciphertext      BYTEA NOT NULL CHECK (octet_length(ciphertext) >= 16),
    key_reference   TEXT NOT NULL CHECK (length(btrim(key_reference)) BETWEEN 1 AND 256),
    schema_version  INTEGER NOT NULL DEFAULT 1 CHECK (schema_version > 0),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by      UUID
);

CREATE TABLE hr.employment_assignments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    employee_id         UUID NOT NULL REFERENCES hr.employees(id) ON DELETE RESTRICT,
    organization_id     UUID NOT NULL REFERENCES org.organizations(id) ON DELETE RESTRICT,
    unit_id             UUID NOT NULL,
    site_id             UUID,
    position_code       TEXT NOT NULL CHECK (position_code = btrim(position_code) AND length(position_code) BETWEEN 1 AND 80),
    assignment_status   TEXT NOT NULL DEFAULT 'active' CHECK (assignment_status IN ('planned', 'active', 'ended', 'cancelled')),
    is_primary          BOOLEAN NOT NULL DEFAULT true,
    valid_from          DATE NOT NULL,
    valid_to            DATE,
    version             BIGINT NOT NULL DEFAULT 1 CHECK (version > 0),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by          UUID,
    CONSTRAINT assignment_validity CHECK (valid_to IS NULL OR valid_to > valid_from),
    CONSTRAINT ended_assignment_has_end_date CHECK (assignment_status <> 'ended' OR valid_to IS NOT NULL),
    CONSTRAINT assignment_unit_org_fk FOREIGN KEY (unit_id, organization_id)
        REFERENCES org.units(id, organization_id) ON DELETE RESTRICT,
    CONSTRAINT assignment_site_unit_fk FOREIGN KEY (site_id, unit_id)
        REFERENCES org.sites(id, unit_id) ON DELETE RESTRICT
);
CREATE INDEX employment_assignments_employee_dates_idx
    ON hr.employment_assignments (employee_id, valid_from DESC, valid_to);
CREATE INDEX employment_assignments_unit_active_idx
    ON hr.employment_assignments (unit_id, valid_from, valid_to)
    WHERE assignment_status = 'active';
CREATE INDEX employment_assignments_site_active_idx
    ON hr.employment_assignments (site_id, valid_from, valid_to)
    WHERE site_id IS NOT NULL AND assignment_status = 'active';

-- Serialize assignment overlap checks per employee, then reject overlapping primary assignments.
-- Date intervals are half-open [valid_from, valid_to), with NULL valid_to meaning unbounded.
CREATE FUNCTION hr.reject_overlapping_primary_assignment()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    IF NEW.is_primary AND NEW.assignment_status <> 'cancelled' THEN
        PERFORM 1 FROM hr.employees WHERE id = NEW.employee_id FOR UPDATE;
        IF EXISTS (
            SELECT 1
            FROM hr.employment_assignments a
            WHERE a.employee_id = NEW.employee_id
              AND a.is_primary
              AND a.assignment_status <> 'cancelled'
              AND a.id <> NEW.id
              AND daterange(a.valid_from, a.valid_to, '[)')
                  && daterange(NEW.valid_from, NEW.valid_to, '[)')
        ) THEN
            RAISE EXCEPTION 'overlapping primary employment assignment for employee %', NEW.employee_id
                USING ERRCODE = '23P01';
        END IF;
    END IF;
    RETURN NEW;
END;
$$;
CREATE TRIGGER employment_assignment_no_primary_overlap
    BEFORE INSERT OR UPDATE OF employee_id, is_primary, assignment_status, valid_from, valid_to
    ON hr.employment_assignments
    FOR EACH ROW EXECUTE FUNCTION hr.reject_overlapping_primary_assignment();

-- Bind actor columns to internal accounts only after both schemas exist.
ALTER TABLE org.organizations
    ADD CONSTRAINT organizations_created_by_fk FOREIGN KEY (created_by) REFERENCES iam.accounts(id) ON DELETE RESTRICT,
    ADD CONSTRAINT organizations_updated_by_fk FOREIGN KEY (updated_by) REFERENCES iam.accounts(id) ON DELETE RESTRICT;
ALTER TABLE org.units
    ADD CONSTRAINT units_created_by_fk FOREIGN KEY (created_by) REFERENCES iam.accounts(id) ON DELETE RESTRICT,
    ADD CONSTRAINT units_updated_by_fk FOREIGN KEY (updated_by) REFERENCES iam.accounts(id) ON DELETE RESTRICT;
ALTER TABLE org.sites
    ADD CONSTRAINT sites_created_by_fk FOREIGN KEY (created_by) REFERENCES iam.accounts(id) ON DELETE RESTRICT,
    ADD CONSTRAINT sites_updated_by_fk FOREIGN KEY (updated_by) REFERENCES iam.accounts(id) ON DELETE RESTRICT;
ALTER TABLE hr.employees
    ADD CONSTRAINT employees_created_by_fk FOREIGN KEY (created_by) REFERENCES iam.accounts(id) ON DELETE RESTRICT,
    ADD CONSTRAINT employees_updated_by_fk FOREIGN KEY (updated_by) REFERENCES iam.accounts(id) ON DELETE RESTRICT;
ALTER TABLE hr.employee_private_profiles
    ADD CONSTRAINT employee_profiles_updated_by_fk FOREIGN KEY (updated_by) REFERENCES iam.accounts(id) ON DELETE RESTRICT;
ALTER TABLE hr.employment_assignments
    ADD CONSTRAINT assignments_created_by_fk FOREIGN KEY (created_by) REFERENCES iam.accounts(id) ON DELETE RESTRICT;

COMMENT ON SCHEMA hr IS 'Minimal workforce registry; sensitive profile fields are isolated and stored as application-encrypted ciphertext.';
COMMENT ON TABLE hr.employees IS 'Internal employee directory keyed by organization-issued code; no national identifier is assumed.';
COMMENT ON TABLE hr.employee_private_profiles IS 'Envelope-encrypted private attributes; ciphertext only, with keys held outside PostgreSQL.';
COMMENT ON TABLE hr.employment_assignments IS 'Effective-dated organization assignment history; overlapping primary periods are rejected transactionally.';

COMMIT;
