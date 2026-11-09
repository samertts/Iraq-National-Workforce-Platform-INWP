import { createServer as createHttpServer } from 'node:http';
import { createServer as createHttpsServer } from 'node:https';
import { readFileSync } from 'node:fs';
import { randomUUID } from 'node:crypto';
import { createPool } from './db.js';
import { createAuthenticator } from './auth.js';
import { insertRejectedAudit } from './audit.js';
import { getEmployee, listEmployees, updateEmployeeStatus } from './employees.js';
import { applySecurityHeaders, enforceSameOrigin, HttpError, sendJson } from './security.js';

const env = process.env;
const production = env.NODE_ENV === 'production';
const required = ['DATABASE_URL', 'OIDC_ISSUER', 'OIDC_AUDIENCE', 'OIDC_JWKS_URL', 'PUBLIC_ORIGIN'];
for (const key of required) if (!env[key]) throw new Error(`${key} is required`);
if (production && !env.DB_CA_PEM) throw new Error('DB_CA_PEM is required in production');
if (production && (!env.TLS_CERT_FILE || !env.TLS_KEY_FILE)) throw new Error('TLS_CERT_FILE and TLS_KEY_FILE are required in production');
if (production && new URL(env.PUBLIC_ORIGIN).protocol !== 'https:') throw new Error('PUBLIC_ORIGIN must use HTTPS in production');
if (new URL(env.OIDC_ISSUER).protocol !== 'https:' || new URL(env.OIDC_JWKS_URL).protocol !== 'https:') {
  throw new Error('OIDC issuer and JWKS URL must use HTTPS');
}

const pool = createPool(env);
const cookieName = env.SESSION_COOKIE_NAME || '__Host-inwp_access';
if (production && !cookieName.startsWith('__Host-')) throw new Error('Production SESSION_COOKIE_NAME must use the __Host- prefix');

async function recordReject(req, id, reason, actorId = null) {
  try {
    await insertRejectedAudit(pool, {
      requestId: id,
      actorId,
      reasonCode: reason,
      method: req.method,
      path: new URL(req.url, 'http://local.invalid').pathname
    });
  } catch (error) {
    console.error(JSON.stringify({ event: 'audit_write_failed', request_id: id, code: error.code || 'unknown' }));
    throw new HttpError(503, 'audit_unavailable');
  }
}

const authenticate = createAuthenticator({
  issuer: env.OIDC_ISSUER,
  audience: env.OIDC_AUDIENCE,
  jwksUrl: env.OIDC_JWKS_URL,
  cookieName,
  db: pool,
  onReject: (req, reason) => recordReject(req, req.inwpRequestId, reason)
});

async function ready() {
  const result = await pool.query(`
    SELECT to_regclass('iam.accounts') IS NOT NULL AS accounts,
           to_regclass('iam.external_identities') IS NOT NULL AS identities,
           to_regclass('iam.role_assignments') IS NOT NULL AS assignments,
           to_regclass('iam.idempotency_records') IS NOT NULL AS idempotency,
           to_regclass('hr.employees') IS NOT NULL AS employees,
           to_regclass('audit.events') IS NOT NULL AS audit
  `);
  return Object.values(result.rows[0]).every(Boolean);
}

async function dispatch(req, res) {
  const id = randomUUID();
  req.inwpRequestId = id;
  applySecurityHeaders(res, id);
  res.setHeader('x-permitted-cross-domain-policies', 'none');
  res.setHeader('permissions-policy', 'camera=(), microphone=(), geolocation=()');
  if (production) res.setHeader('strict-transport-security', 'max-age=31536000; includeSubDomains');

  try {
    const url = new URL(req.url, 'http://local.invalid');
    if (req.method === 'GET' && url.pathname === '/healthz') {
      return sendJson(res, 200, { status: 'ok', service: 'inwp-workforce-api' });
    }
    if (req.method === 'GET' && url.pathname === '/readyz') {
      if (!(await ready())) return sendJson(res, 503, { error: 'schema_not_ready', request_id: id });
      return sendJson(res, 200, { status: 'ready' });
    }

    if (!['GET', 'PATCH'].includes(req.method)) throw new HttpError(405, 'method_not_allowed');
    if (req.method !== 'GET') enforceSameOrigin(req, env.PUBLIC_ORIGIN);
    const account = await authenticate(req);
    if (!account) return sendJson(res, 401, { error: 'authentication_required', request_id: id });
    req.inwpAccount = account;

    if (req.method === 'GET' && url.pathname === '/v1/employees') {
      return await listEmployees(pool, account, id, url, res);
    }
    const match = /^\/v1\/employees\/([0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12})$/i.exec(url.pathname);
    if (match && req.method === 'GET') return await getEmployee(pool, account, id, match[1], res);
    if (match && req.method === 'PATCH') return await updateEmployeeStatus(pool, account, id, req, res, match[1]);
    throw new HttpError(404, 'route_not_found');
  } catch (error) {
    const status = error instanceof HttpError ? error.status : 500;
    if (!error.auditRecorded && status >= 400 && status < 500) {
      try { await recordReject(req, id, error.code || 'request_rejected', req.inwpAccount?.accountId || null); }
      catch { return sendJson(res, 503, { error: 'audit_unavailable', request_id: id }); }
    }
    if (!(error instanceof HttpError)) {
      console.error(JSON.stringify({ event: 'request_failed', request_id: id, code: error.code || 'internal_error' }));
      try { await recordReject(req, id, 'internal_error', req.inwpAccount?.accountId || null); }
      catch { console.error(JSON.stringify({ event: 'failure_audit_unavailable', request_id: id })); }
    }
    return sendJson(res, status, {
      error: error instanceof HttpError ? error.code : 'internal_error',
      request_id: id
    });
  }
}

const handler = (req, res) => { void dispatch(req, res); };
const host = env.HOST || (production ? '0.0.0.0' : '127.0.0.1');
const port = Number(env.PORT || 8080);
const server = production
  ? createHttpsServer({
      key: readFileSync(env.TLS_KEY_FILE),
      cert: readFileSync(env.TLS_CERT_FILE),
      minVersion: 'TLSv1.2'
    }, handler)
  : createHttpServer(handler);

server.requestTimeout = 15000;
server.headersTimeout = 10000;
server.keepAliveTimeout = 5000;
server.maxHeadersCount = 64;
server.listen(port, host, () => {
  console.log(JSON.stringify({ event: 'server_started', host, port, tls: production }));
});

async function shutdown(signal) {
  console.log(JSON.stringify({ event: 'server_shutdown', signal }));
  server.close(async () => {
    await pool.end();
    process.exit(0);
  });
  setTimeout(() => process.exit(1), 10000).unref();
}
process.on('SIGTERM', () => void shutdown('SIGTERM'));
process.on('SIGINT', () => void shutdown('SIGINT'));
