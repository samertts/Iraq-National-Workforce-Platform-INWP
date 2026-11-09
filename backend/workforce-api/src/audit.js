export async function insertAudit(client, event) {
  const actorKind = event.actorId ? 'user' : 'system';
  await client.query(
    `INSERT INTO audit.events
       (actor_account_id, actor_kind, action, entity_schema, entity_table, entity_id,
        outcome, reason_code, request_id, before_state, after_state, metadata)
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::jsonb, $11::jsonb, $12::jsonb)`,
    [
      event.actorId || null,
      actorKind,
      event.action,
      event.entitySchema,
      event.entityTable,
      event.entityId || null,
      event.outcome || 'succeeded',
      event.reasonCode || null,
      event.requestId,
      event.beforeState == null ? null : JSON.stringify(event.beforeState),
      event.afterState == null ? null : JSON.stringify(event.afterState),
      JSON.stringify(event.metadata || {})
    ]
  );
}

export async function insertRejectedAudit(pool, { requestId, actorId = null, reasonCode, method, path }) {
  const client = await pool.connect();
  try {
    await insertAudit(client, {
      actorId,
      action: 'security.access_denied',
      entitySchema: 'api',
      entityTable: 'requests',
      outcome: 'rejected',
      reasonCode,
      requestId,
      metadata: { method, path }
    });
  } finally {
    client.release();
  }
}
