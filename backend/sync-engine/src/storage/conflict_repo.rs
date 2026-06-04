use crate::error::SyncResult;
use crate::reconciliation::QuarantineEntry;
use sqlx::PgPool;

#[derive(Clone)]
pub struct ConflictRepo {
    pool: PgPool,
}

impl ConflictRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, entry: &QuarantineEntry) -> SyncResult<()> {
        sqlx::query(
            r#"
            INSERT INTO sync.sync_conflicts
                (conflict_id, partition_key, record_id, record_type,
                 local_version, remote_version, strategy, status,
                 escalated_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (conflict_id) DO NOTHING
            "#,
        )
        .bind(entry.conflict_id)
        .bind(&entry.partition_key)
        .bind(&entry.record_id)
        .bind(&entry.record_type)
        .bind(&entry.local_payload)
        .bind(&entry.remote_payload)
        .bind(&entry.strategy)
        .bind(format!("{:?}", entry.status).to_lowercase())
        .bind(entry.escalated_at)
        .bind(entry.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn resolve(
        &self,
        conflict_id: uuid::Uuid,
        resolution: &str,
        resolved_by: uuid::Uuid,
    ) -> SyncResult<()> {
        sqlx::query(
            r#"
            UPDATE sync.sync_conflicts
            SET status = 'resolved', resolution = $1, resolved_by = $2, resolved_at = $3
            WHERE conflict_id = $4
            "#,
        )
        .bind(resolution)
        .bind(resolved_by)
        .bind(chrono::Utc::now())
        .bind(conflict_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_open(&self, limit: i64) -> SyncResult<Vec<QuarantineEntry>> {
        let rows = sqlx::query_as::<_, ConflictRow>(
            "SELECT * FROM sync.sync_conflicts WHERE status = 'open' ORDER BY created_at ASC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    pub async fn count_open(&self) -> SyncResult<i64> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sync.sync_conflicts WHERE status = 'open'")
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0)
    }
}

#[derive(sqlx::FromRow)]
struct ConflictRow {
    conflict_id: uuid::Uuid,
    partition_key: String,
    record_id: String,
    record_type: String,
    local_version: Vec<u8>,
    remote_version: Vec<u8>,
    strategy: String,
    status: String,
    escalated_at: Option<chrono::DateTime<chrono::Utc>>,
    resolved_by: Option<uuid::Uuid>,
    resolution: Option<String>,
    resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<ConflictRow> for QuarantineEntry {
    type Error = crate::error::SyncEngineError;

    fn try_from(row: ConflictRow) -> Result<Self, Self::Error> {
        Ok(QuarantineEntry {
            conflict_id: row.conflict_id,
            partition_key: row.partition_key,
            record_id: row.record_id,
            record_type: row.record_type,
            local_payload: row.local_version,
            remote_payload: row.remote_version,
            strategy: row.strategy,
            status: crate::reconciliation::QuarantineStatus::Open,
            escalated_at: row.escalated_at.unwrap_or(row.created_at),
            resolved_by: row.resolved_by,
            resolution: row.resolution,
            resolved_at: row.resolved_at,
            created_at: row.created_at,
        })
    }
}
