# INWP database migrations (initial foundation)

These migrations create the initial `org`, `iam`, `hr`, and append-only `audit` schemas plus API idempotency receipts. They are intended for a clean PostgreSQL 13+ development database and contain no seed or production data.

## Apply order

Run the `*.up.sql` files in numeric order, each as a single migration:

1. `001_org.up.sql`
2. `002_iam.up.sql`
3. `003_hr.up.sql`
4. `004_audit.up.sql`
5. `005_api_idempotency.up.sql`

Each file manages its own transaction. Record each successfully applied migration in the deployment's migration ledger. Do not rerun a partially failed migration blindly; inspect transaction state and migration history first.

## Rollback order

Rollback is destructive and deletes all data in the affected schemas. Only use it in a disposable development environment or after an explicitly approved, verified backup. Roll back in reverse order: `005_api_idempotency.down.sql`, `004_audit.down.sql`, `003_hr.down.sql`, `002_iam.down.sql`, `001_org.down.sql`.

## Security boundaries not completed by these files

- Authentication is delegated to an approved institutional OIDC/SAML identity provider. No password table is created.
- `hr.employee_private_profiles.ciphertext` must be envelope-encrypted by a trusted service using a managed key. The database stores neither plaintext private attributes nor encryption keys; the migration does not encrypt input automatically.
- The migrations intentionally do not enable RLS yet. Tenant and organizational scope policies must be designed against the approved authorization model, established by a trusted backend transaction context, and tested with a non-owner runtime role before any real employee records are loaded. RLS is defense in depth, not a replacement for service authorization.
- Do not run the application as schema/table owner, superuser, or a role with `BYPASSRLS`. Separate migration-owner and runtime roles and grant only required operations in a later reviewed deployment migration.
- `audit.events` rejects UPDATE, DELETE, and TRUNCATE through triggers, but a database owner/superuser can bypass this protection. Use restricted writer/reader roles and ship an independently retained copy to a tamper-resistant audit destination. Keep before/after JSON allowlisted and redacted; never log credentials, tokens, or unnecessary sensitive attributes.
- Actor UUIDs and `version` fields do not automatically produce audit entries or optimistic-lock enforcement. Every application write must validate the expected version and insert its audit event in the same transaction. The application owns version increments and state-machine rules.
- Primary assignment overlap is guarded by a trigger that locks the employee row before checking date ranges. Exercise concurrent insertion/update tests on the target PostgreSQL version.
- The schema leaves employee identifiers, lawful fields, hierarchy semantics, retention, and role vocabulary to the data owner; it deliberately does not invent national IDs or seed privileged roles.

## Verification before operational use

Run SQL lint/static checks, apply all `up` migrations to a disposable PostgreSQL 13+ instance, verify reverse-order rollback, and add integration tests for FK/unique/check failures, concurrent assignment overlap, audit mutation rejection, stale version rejection, role scope authorization, and cross-organization isolation. Production use remains blocked until RLS/grants, institutional identity integration, retention/legal approval, backup/restore tests, and independent security review are complete.
