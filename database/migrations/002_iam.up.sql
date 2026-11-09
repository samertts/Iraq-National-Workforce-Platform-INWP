-- INWP identity and access-control foundation.
-- Authentication credentials remain with the configured external IdP; no passwords are stored here.
BEGIN;

CREATE SCHEMA iam;

CREATE TABLE iam.accounts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    status          TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'active', 'suspended', 'closed')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    disabled_at     TIMESTAMPTZ,
    version         BIGINT NOT NULL DEFAULT 1 CHECK (version > 0),
    CONSTRAINT accounts_status_disabled_consistency CHECK (
        (status IN ('suspended', 'closed') AND disabled_at IS NOT NULL)
        OR (status IN ('pending', 'active') AND disabled_at IS NULL)
    )
);

CREATE TABLE iam.external_identities (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL REFERENCES iam.accounts(id) ON DELETE RESTRICT,
    issuer          TEXT NOT NULL CHECK (length(issuer) BETWEEN 1 AND 512),
    subject         TEXT NOT NULL CHECK (length(subject) BETWEEN 1 AND 512),
    linked_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at    TIMESTAMPTZ,
    UNIQUE (issuer, subject),
    UNIQUE (account_id, issuer)
);
CREATE INDEX external_identities_account_idx ON iam.external_identities (account_id);

CREATE TABLE iam.roles (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    role_key        TEXT NOT NULL UNIQUE CHECK (role_key ~ '^[a-z][a-z0-9_.:-]{1,95}$'),
    name            TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 160),
    description     TEXT,
    is_system       BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE iam.permissions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    permission_key  TEXT NOT NULL UNIQUE CHECK (permission_key ~ '^[a-z][a-z0-9_.:-]{1,127}$'),
    description     TEXT NOT NULL CHECK (length(description) BETWEEN 1 AND 500),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE iam.role_permissions (
    role_id         UUID NOT NULL REFERENCES iam.roles(id) ON DELETE RESTRICT,
    permission_id   UUID NOT NULL REFERENCES iam.permissions(id) ON DELETE RESTRICT,
    granted_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (role_id, permission_id)
);
CREATE INDEX role_permissions_permission_idx ON iam.role_permissions (permission_id);

-- A grant may be global, organization-scoped, unit-scoped, or site-scoped.
-- Authorization must still be checked in trusted application services for every operation.
CREATE TABLE iam.role_assignments (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id          UUID NOT NULL REFERENCES iam.accounts(id) ON DELETE RESTRICT,
    role_id             UUID NOT NULL REFERENCES iam.roles(id) ON DELETE RESTRICT,
    organization_id     UUID REFERENCES org.organizations(id) ON DELETE RESTRICT,
    unit_id             UUID REFERENCES org.units(id) ON DELETE RESTRICT,
    site_id             UUID REFERENCES org.sites(id) ON DELETE RESTRICT,
    granted_by          UUID NOT NULL REFERENCES iam.accounts(id) ON DELETE RESTRICT,
    grant_reason        TEXT NOT NULL CHECK (length(btrim(grant_reason)) BETWEEN 1 AND 1000),
    valid_from          TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_until         TIMESTAMPTZ,
    revoked_at          TIMESTAMPTZ,
    revoked_by          UUID REFERENCES iam.accounts(id) ON DELETE RESTRICT,
    revocation_reason   TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT role_assignment_validity CHECK (valid_until IS NULL OR valid_until > valid_from),
    CONSTRAINT role_assignment_revocation CHECK (
        (revoked_at IS NULL AND revoked_by IS NULL AND revocation_reason IS NULL)
        OR (revoked_at IS NOT NULL AND revoked_by IS NOT NULL
            AND revocation_reason IS NOT NULL AND length(btrim(revocation_reason)) > 0)
    ),
    CONSTRAINT role_assignment_single_scope CHECK (num_nonnulls(organization_id, unit_id, site_id) <= 1)
);
CREATE INDEX role_assignments_account_active_idx
    ON iam.role_assignments (account_id, valid_from, valid_until) WHERE revoked_at IS NULL;
CREATE INDEX role_assignments_role_idx ON iam.role_assignments (role_id);
CREATE INDEX role_assignments_org_idx ON iam.role_assignments (organization_id) WHERE organization_id IS NOT NULL;
CREATE INDEX role_assignments_unit_idx ON iam.role_assignments (unit_id) WHERE unit_id IS NOT NULL;
CREATE INDEX role_assignments_site_idx ON iam.role_assignments (site_id) WHERE site_id IS NOT NULL;

-- Store only a one-way digest of refresh tokens if the application requires server-side sessions.
-- Never store bearer or refresh token values in plaintext.
CREATE TABLE iam.refresh_sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL REFERENCES iam.accounts(id) ON DELETE RESTRICT,
    token_digest    BYTEA NOT NULL UNIQUE CHECK (octet_length(token_digest) = 32),
    issued_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL,
    rotated_at      TIMESTAMPTZ,
    revoked_at      TIMESTAMPTZ,
    revoke_reason   TEXT,
    CONSTRAINT refresh_session_expiry CHECK (expires_at > issued_at),
    CONSTRAINT refresh_session_revocation CHECK (revoked_at IS NULL OR revoked_at >= issued_at)
);
CREATE INDEX refresh_sessions_account_active_idx
    ON iam.refresh_sessions (account_id, expires_at) WHERE revoked_at IS NULL;

COMMENT ON SCHEMA iam IS 'Internal account mapping and authorization metadata; external IdP owns authentication credentials.';
COMMENT ON TABLE iam.external_identities IS 'Maps an OIDC/SAML issuer and subject to an internal account; subject values are restricted identifiers.';
COMMENT ON TABLE iam.role_assignments IS 'Role grants are temporal and optionally scoped; application authorization is mandatory.';
COMMENT ON TABLE iam.refresh_sessions IS 'Optional server-side refresh-session registry; stores SHA-256 token digests only, never raw tokens.';

COMMIT;
