import { createHash, timingSafeEqual } from 'node:crypto';
import { randomUUID } from 'node:crypto';
import { insertAudit } from './audit.js';
import { HttpError, readJson, sendJson } from './security.js';

const ACTIVE_ASSIGNMENT = `a.employee_id = e.id
  AND a.is_primary = true
  AND a.assignment_status = 'active'
  AND a.valid_from <= CURRENT_DATE
  AND (a.valid_to IS NULL OR a.valid_to > CURRENT_DATE)`;

const HAS_PERMISSION = `EXISTS (
  SELECT 1
    FROM iam.role_assignments ra
    JOIN iam.role_permissions rp ON rp.role_id = ra.role_id
    JOIN iam.permissions p ON p.id = rp.permission_id
   WHERE ra.account_id = $1
     AND p.permission_key = $2
     AND ra.revoked_at IS NULL
     AND ra.valid_from <= now()
     AND (ra.valid_until IS NULL OR ra.valid_until > now())
)`;

const EMPLOYEE_SCOPE = `EXISTS (
  SELECT 1
    FROM iam.role_assignments ra
    JOIN iam.role_permissions rp ON rp.role_id = ra.role_id
    JOIN iam.permissions p ON p.id = rp.permission_id
   WHERE ra.account_id = $1
     AND p.permission_key = $2
     AND ra.revoked_at IS NULL
     AND ra.valid_from <= now()
     AND (ra.valid_until IS NULL OR ra.valid_until > now())
     AND (
       num_nonnulls(ra.organization_id, ra.unit_id, ra.site_id) = 0
       OR ra.organization_id = e.registry_org_id
       OR EXISTS (
         SELECT 1 FROM hr.employment_assignments a
          WHERE ${ACTIVE_ASSIGNMENT}
            AND a.organization_id = ra.organization_id
       )
       OR EXISTS (
         SELECT 1 FROM hr.employment_assignments a
          WHERE ${ACTIVE_ASSIGNMENT}
            AND a.unit_id = ra.unit_id
       )
       OR EXISTS (
         SELECT 1 FROM hr.employment_assignments a
          WHERE ${ACTIVE_ASSIGNMENT}
            AND a.site_id = ra.site_id
       )
     )
)`;

export async function hasPermission(client, accountId, permission) {
  const result = await client.query(`SELECT ${HAS_PERMISSION} AS allowed`, [accountId, permission]);
  return result.rows[0]?.allowed === true;
}

function employeeView(row) {
  return {
    id: row.id,
    organization_id: row.registry_org_id,
    employee_code: row.employee_code,
    employment_status: row.employment_status,
    hired_on: row.hired_on,
    separated_on: row.separated_on,
    version: Number(row.version),
    updated_at: row.updated_at
  };
}

export async function listEmployees(pool, account, requestId, url, res) {
  const limit = Number(url.searchParams.get('limit') || 25);
  const offset = Number(url.searchParams.get('offset') || 0);
  if (!Number.isInteger(limit) || limit < 1 || limit > 100 || !Number.isInteger(offset) || offset < 0 || offset > 100000) {
    throw new HttpError(400, 'invalid_pagination');
  }
  const client = await pool.connect();
  try {
    await client.query('BEGIN ISOLATION LEVEL REPEATABLE READ');
    if (!(await hasPermission(client, account.accountId, 'hr.employee.read'))) {
      await insertAudit(client, {
        actorId: account.accountId, action: 'security.authorization_denied',
        entitySchema: 'hr', entityTable: 'employees', outcome: 'rejected',
        reasonCode: 'permission_missing', requestId, metadata: { operation: 'list' }
      });
      await client.query('COMMIT');
      const error = new HttpError(403, 'permission_denied');
      error.auditRecorded = true;
      throw error;
    }
    const result = await client.query(
      `SELECT e.id, e.registry_org_id, e.employee_code, e.employment_status,
              e.hired_on, e.separated_on, e.version, e.updated_at
         FROM hr.employees e
        WHERE ${EMPLOYEE_SCOPE}
        ORDER BY e.id
        LIMIT $3 OFFSET $4`,
      [account.accountId, 'hr.employee.read', limit, offset]
    );
    await insertAudit(client, {
      actorId: account.accountId, action: 'hr.employee.list',
      entitySchema: 'hr', entityTable: 'employees', requestId,
      metadata: { result_count: result.rowCount, limit, offset }
    });
    await client.query('COMMIT');
    sendJson(res, 200, { employees: result.rows.map(employeeView), limit, offset });
  } catch (error) {
    try { await client.query('ROLLBACK'); } catch {}
    throw error;
  } finally {
    client.release();
  }
}

export async function getEmployee(pool, account, requestId, employeeId, res) {
  const client = await pool.connect();
  try {
    await client.query('BEGIN ISOLATION LEVEL REPEATABLE READ');
    if (!(await hasPermission(client, account.accountId, 'hr.employee.read'))) {
      await insertAudit(client, {
        actorId: account.accountId, action: 'security.authorization_denied',
        entitySchema: 'hr', entityTable: 'employees', entityId: employeeId,
        outcome: 'rejected', reasonCode: 'permission_missing', requestId
      });
      await client.query('COMMIT');
      const error = new HttpError(403, 'permission_denied');
      error.auditRecorded = true;
      throw error;
    }
    const result = await client.query(
      `SELECT e.id, e.registry_org_id, e.employee_code, e.employment_status,
              e.hired_on, e.separated_on, e.version, e.updated_at
         FROM hr.employees e
        WHERE e.id = $3 AND ${EMPLOYEE_SCOPE}
        LIMIT 1`,
      [account.accountId, 'hr.employee.read', employeeId]
    );
    if (!result.rows[0]) {
      await insertAudit(client, {
        actorId: account.accountId, action: 'security.authorization_denied',
        entitySchema: 'hr', entityTable: 'employees', entityId: employeeId,
        outcome: 'rejected', reasonCode: 'resource_not_found_or_out_of_scope', requestId
      });
      await client.query('COMMIT');
      const error = new HttpError(404, 'employee_not_found');
      error.auditRecorded = true;
      throw error;
    }
    await insertAudit(client, {
      actorId: account.accountId, action: 'hr.employee.read',
      entitySchema: 'hr', entityTable: 'employees', entityId: employeeId,
      requestId, metadata: {}
    });
    await client.query('COMMIT');
    sendJson(res, 200, { employee: employeeView(result.rows[0]) });
  } catch (error) {
    try { await client.query('ROLLBACK'); } catch {}
    throw error;
  } finally {
    client.release();
  }
}

const ALLOWED_TRANSITIONS = new Map([
  ['pending', new Set(['active', 'suspended'])],
  ['active', new Set(['on_leave', 'suspended', 'separated'])],
  ['on_leave', new Set(['active', 'suspended', 'separated'])],
  ['suspended', new Set(['active', 'separated'])],
  ['separated', new Set()]
]);

function parseExpectedVersion(header) {
  if (!header) throw new HttpError(428, 'if_match_required');
  const match = /^(?:"([1-9][0-9]*)"|([1-9][0-9]*))$/.exec(header.trim());
  if (!match) throw new HttpError(400, 'invalid_if_match');
  const value = Number(match[1] || match[2]);
  if (!Number.isSafeInteger(value)) throw new HttpError(400, 'invalid_if_match');
  return value;
}

function digestRequest(path, status, expectedVersion) {
  return createHash('sha256')
    .update(`PATCH\n${path}\n${status}\n${expectedVersion}`, 'utf8')
    .digest();
}

export async function updateEmployeeStatus(pool, account, requestId, req, res, employeeId) {
  const expectedVersion = parseExpectedVersion(req.headers['if-match']);
  const idempotencyKey = req.headers['idempotency-key'];
  if (typeof idempotencyKey !== 'string' || !/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i.test(idempotencyKey)) {
    throw new HttpError(400, 'valid_idempotency_key_required');
  }
  const body = await readJson(req);
  if (Object.keys(body).length !== 1 || !['pending', 'active', 'on_leave', 'suspended', 'separated'].includes(body.employment_status)) {
    throw new HttpError(400, 'invalid_employee_status_request');
  }
  const nextStatus = body.employment_status;
  const digest = digestRequest(`/v1/employees/${employeeId}`, nextStatus, expectedVersion);
  const client = await pool.connect();
  try {
    await client.query('BEGIN ISOLATION LEVEL SERIALIZABLE');
    await client.query("SELECT set_config('app.current_account_id', $1, true)", [account.accountId]);

    if (!(await hasPermission(client, account.accountId, 'hr.employee.update'))) {
      await insertAudit(client, {
        actorId: account.accountId, action: 'security.authorization_denied',
        entitySchema: 'hr', entityTable: 'employees', entityId: employeeId,
        outcome: 'rejected', reasonCode: 'permission_missing', requestId
      });
      await client.query('COMMIT');
      const error = new HttpError(403, 'permission_denied');
      error.auditRecorded = true;
      throw error;
    }

    const currentResult = await client.query(
      `SELECT e.id, e.registry_org_id, e.employee_code, e.employment_status,
              e.hired_on, e.separated_on, e.version, e.updated_at
         FROM hr.employees e
        WHERE e.id = $3 AND ${EMPLOYEE_SCOPE}
        FOR UPDATE OF e`,
      [account.accountId, 'hr.employee.update', employeeId]
    );
    const current = currentResult.rows[0];
    if (!current) {
      await insertAudit(client, {
        actorId: account.accountId, action: 'security.authorization_denied',
        entitySchema: 'hr', entityTable: 'employees', entityId: employeeId,
        outcome: 'rejected', reasonCode: 'resource_not_found_or_out_of_scope', requestId
      });
      await client.query('COMMIT');
      const error = new HttpError(404, 'employee_not_found');
      error.auditRecorded = true;
      throw error;
    }

    await client.query(
      `INSERT INTO iam.idempotency_records (account_id, idempotency_key, request_digest)
       VALUES ($1, $2, $3)
       ON CONFLICT (account_id, idempotency_key) DO NOTHING`,
      [account.accountId, idempotencyKey, digest]
    );
    const receiptResult = await client.query(
      `SELECT request_digest, response_status, response_body, completed_at
         FROM iam.idempotency_records
        WHERE account_id = $1 AND idempotency_key = $2
        FOR UPDATE`,
      [account.accountId, idempotencyKey]
    );
    const receipt = receiptResult.rows[0];
    if (!receipt || !timingSafeEqual(Buffer.from(receipt.request_digest), digest)) {
      await client.query('ROLLBACK');
      throw new HttpError(409, 'idempotency_key_reused');
    }
    if (receipt.completed_at) {
      const replay = receipt.response_body;
      const status = receipt.response_status;
      await insertAudit(client, {
        actorId: account.accountId, action: 'api.idempotency_replay',
        entitySchema: 'hr', entityTable: 'employees', entityId: employeeId,
        requestId, metadata: { idempotency_key: idempotencyKey }
      });
      await client.query('COMMIT');
      sendJson(res, status, replay);
      return;
    }

    if (Number(current.version) !== expectedVersion) {
      await client.query('ROLLBACK');
      throw new HttpError(412, 'version_conflict');
    }
    if (!ALLOWED_TRANSITIONS.get(current.employment_status)?.has(nextStatus)) {
      await client.query('ROLLBACK');
      throw new HttpError(409, 'invalid_state_transition');
    }
    if (nextStatus === 'active' && !current.hired_on) {
      await client.query('ROLLBACK');
      throw new HttpError(409, 'hire_date_required_before_activation');
    }

    const updatedResult = await client.query(
      `UPDATE hr.employees
          SET employment_status = $2,
              separated_on = CASE WHEN $2 = 'separated' THEN CURRENT_DATE ELSE separated_on END,
              updated_at = now(), updated_by = $3, version = version + 1
        WHERE id = $1 AND version = $4
        RETURNING id, registry_org_id, employee_code, employment_status,
                  hired_on, separated_on, version, updated_at`,
      [employeeId, nextStatus, account.accountId, expectedVersion]
    );
    if (!updatedResult.rows[0]) {
      await client.query('ROLLBACK');
      throw new HttpError(412, 'version_conflict');
    }
    const updated = updatedResult.rows[0];
    const beforeState = { id: current.id, status: current.employment_status, version: Number(current.version) };
    const afterState = { id: updated.id, status: updated.employment_status, version: Number(updated.version) };
    const responseBody = { employee_id: updated.id, employment_status: updated.employment_status, version: Number(updated.version) };

    await insertAudit(client, {
      actorId: account.accountId, action: 'hr.employee.status_changed',
      entitySchema: 'hr', entityTable: 'employees', entityId: employeeId,
      requestId, beforeState, afterState,
      metadata: { idempotency_key: idempotencyKey }
    });
    await client.query(
      `UPDATE iam.idempotency_records
          SET resource_id = $3, response_status = 200, response_body = $4::jsonb, completed_at = now()
        WHERE account_id = $1 AND idempotency_key = $2`,
      [account.accountId, idempotencyKey, employeeId, JSON.stringify(responseBody)]
    );
    await client.query('COMMIT');
    sendJson(res, 200, responseBody);
  } catch (error) {
    try { await client.query('ROLLBACK'); } catch {}
    throw error;
  } finally {
    client.release();
  }
}

export function newEmployeeId() { return randomUUID(); }
