use crate::error::SyncResult;
use sqlx::PgPool;

#[derive(Clone)]
pub struct EventStore {
    pool: PgPool,
}

impl EventStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn append(&self, event: &crate::events::contract::SyncEvent) -> SyncResult<()> {
        sqlx::query(
            r#"
            INSERT INTO sync.event_store
                (event_id, node_id, event_type, partition_key, payload,
                 version_vector, local_timestamp, signature, signing_key_id,
                 schema_version, metadata, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (event_id) DO NOTHING
            "#,
        )
        .bind(event.event_id)
        .bind(event.node_id)
        .bind(&event.event_type)
        .bind(&event.partition_key)
        .bind(&event.payload)
        .bind(serde_json::to_value(&event.version_vector)?)
        .bind(event.local_timestamp as i64)
        .bind(&event.signature)
        .bind(&event.signing_key_id)
        .bind(&event.schema_version)
        .bind(serde_json::to_value(&event.metadata)?)
        .bind(event.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn append_batch(
        &self,
        events: &[crate::events::contract::SyncEvent],
    ) -> SyncResult<()> {
        for event in events {
            self.append(event).await?;
        }
        Ok(())
    }

    pub async fn replay_from(
        &self,
        partition_key: &str,
        since: chrono::DateTime<chrono::Utc>,
        limit: i64,
    ) -> SyncResult<Vec<crate::events::contract::SyncEvent>> {
        let rows = sqlx::query_as::<_, EventRow>(
            r#"
            SELECT * FROM sync.event_store
            WHERE partition_key = $1 AND created_at >= $2
            ORDER BY created_at ASC
            LIMIT $3
            "#,
        )
        .bind(partition_key)
        .bind(since)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    pub async fn replay_range(
        &self,
        partition_key: &str,
        from_seq: i64,
        to_seq: i64,
    ) -> SyncResult<Vec<crate::events::contract::SyncEvent>> {
        let rows = sqlx::query_as::<_, EventRow>(
            r#"
            SELECT * FROM sync.event_store
            WHERE partition_key = $1
              AND event_seq >= $2 AND event_seq <= $3
            ORDER BY event_seq ASC
            "#,
        )
        .bind(partition_key)
        .bind(from_seq)
        .bind(to_seq)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    pub async fn count_since(
        &self,
        partition_key: &str,
        since: chrono::DateTime<chrono::Utc>,
    ) -> SyncResult<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sync.event_store WHERE partition_key = $1 AND created_at >= $2",
        )
        .bind(partition_key)
        .bind(since)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    pub async fn purge_before(&self, before: chrono::DateTime<chrono::Utc>) -> SyncResult<u64> {
        let result = sqlx::query("DELETE FROM sync.event_store WHERE created_at < $1")
            .bind(before)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

#[derive(sqlx::FromRow)]
struct EventRow {
    event_id: uuid::Uuid,
    event_seq: i64,
    node_id: uuid::Uuid,
    event_type: String,
    partition_key: String,
    payload: Vec<u8>,
    version_vector: serde_json::Value,
    local_timestamp: i64,
    signature: Vec<u8>,
    signing_key_id: String,
    schema_version: String,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<EventRow> for crate::events::contract::SyncEvent {
    type Error = crate::error::SyncEngineError;

    fn try_from(row: EventRow) -> Result<Self, Self::Error> {
        Ok(crate::events::contract::SyncEvent {
            event_id: row.event_id,
            node_id: row.node_id,
            event_type: row.event_type,
            partition_key: row.partition_key,
            payload: row.payload,
            version_vector: serde_json::from_value(row.version_vector)?,
            local_timestamp: row.local_timestamp as u64,
            signature: row.signature,
            signing_key_id: row.signing_key_id,
            schema_version: row.schema_version,
            metadata: serde_json::from_value(row.metadata).unwrap_or_default(),
            created_at: row.created_at,
        })
    }
}
