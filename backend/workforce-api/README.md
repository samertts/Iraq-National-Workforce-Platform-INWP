# INWP Workforce API (initial secure skeleton)

This isolated Node.js service exposes a deliberately small employee API. It reads the `iam`, `hr`, and `audit` schemas created by the ordered database migrations. It does not create accounts, seed privileged roles, enroll identity providers, or write private-profile ciphertext.

## Implemented routes

- `GET /healthz`: process liveness only.
- `GET /readyz`: verifies required IAM, HR, idempotency, and audit tables are present.
- `GET /v1/employees?limit=25&offset=0`: returns a bounded page of minimum employee-directory fields, after checking `hr.employee.read` and organization/unit/site scope.
- `GET /v1/employees/{uuid}`: returns one in-scope employee directory record.
- `PATCH /v1/employees/{uuid}`: changes only `employment_status`; requires `If-Match` version, UUID `Idempotency-Key`, exact same-origin `Origin`, and permission `hr.employee.update`. State transitions are allowlisted. The update, audit event, and idempotency receipt share one PostgreSQL transaction. Replaying the same idempotency key returns the saved minimal response and creates a separate replay audit event.

All SQL values use PostgreSQL `$n` parameters. Table/column names and SQL fragments are fixed constants, never assembled from user input. Responses omit `hr.employee_private_profiles` and do not return external identity or authorization data.

## Authentication boundary

Every API route other than health/readiness requires the `__Host-inwp_access` cookie (configurable by `SESSION_COOKIE_NAME`). Its JWT is verified in the service against the configured OIDC issuer, API audience, and HTTPS JWKS endpoint, including signature, algorithm allowlist, expiry, issue time, and a five-minute maximum token age. The verified `(iss, sub)` pair is mapped to an active `iam.accounts` row. Role and scope claims from the token are ignored; active permissions and scope are evaluated from PostgreSQL on every employee query.

This API **does not implement the interactive OIDC authorization-code/PKCE login flow, set the cookie, or rotate refresh tokens**. A separately reviewed identity/BFF component must issue the short-lived cookie with `Secure; HttpOnly; SameSite=Lax; Path=/` and `__Host-` requirements, handle login/PKCE and refresh-token rotation, and protect refresh credentials. The service rejects bearer-header-only authentication. Do not forward unverified identity headers from a proxy.

Unsafe requests require an exact configured `PUBLIC_ORIGIN`; no permissive CORS is enabled. Production mode serves HTTPS directly using mounted TLS cert/key files and requires a PostgreSQL CA certificate. In production, use private networking and TLS between every service hop.

## Required environment

- `NODE_ENV=production` for deployed mode.
- `DATABASE_URL`: secret-managed PostgreSQL connection URL for a dedicated runtime principal.
- `DB_CA_PEM`: trusted CA certificate for PostgreSQL TLS in production.
- `OIDC_ISSUER`, `OIDC_AUDIENCE`, `OIDC_JWKS_URL`: approved institutional provider settings; audience must be specific to this API.
- `PUBLIC_ORIGIN`: exact browser origin, HTTPS in production.
- `TLS_CERT_FILE`, `TLS_KEY_FILE`: mounted TLS files in production.
- Optional `HOST`, `PORT`, `DB_POOL_MAX`, `SESSION_COOKIE_NAME`.

Never put values or private keys in source control. The API assumes migrations have already been applied in order, including `005_api_idempotency.up.sql`.

## Authorization and operational gates

Permission identifiers expected in the database are `hr.employee.read` and `hr.employee.update`. No permissions or roles are seeded; a reviewed administrative process must provision them and scope grants. The query enforces global, organization, unit, or site grants against employee registry and current primary assignments. Confirm this policy against the officially approved organizational model before use.

The initial migrations do not yet enable PostgreSQL RLS or define least-privilege SQL grants. Therefore this service is **not production-ready** until a restricted non-owner database role and independently tested RLS policies are applied. Do not use the schema/table owner or a `BYPASSRLS` role as the API principal. Also require: real PostgreSQL integration tests (including cross-organization denial and concurrent version/assignment conflicts), approved employee field/retention rules, independent security review, backup/restore rehearsal, rate limiting at a trusted gateway, and an external immutable audit sink. Database triggers protect audit rows from ordinary UPDATE/DELETE/TRUNCATE but cannot constrain a superuser/database owner.

No offline client cache or synchronization is implemented here. Sensitive workforce data must not be put into browser storage until a separate threat model and device-wipe/expiry/reconciliation design is approved. This workforce service is not a clinical records service; sample-verification rules are not relevant. General fail-closed, data-integrity, traceability, least-privilege, and recovery principles remain applicable.

## Local checks

```sh
npm ci
npm run check
npm test
npm start
```

Local HTTP binds to `127.0.0.1` by default. No real employee records are included. A PostgreSQL instance, approved OIDC provider settings, schema migrations, roles, and grants are required for endpoint integration tests and actual use.
