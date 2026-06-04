use crate::core::types::{BatchReceipt, SyncDirection};
use crate::error::SyncResult;
use sqlx::PgPool;

#[derive(Clone)]
pub struct BatchLogRepo {
    pool: PgPool,
}

impl BatchLogRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, receipt: &BatchReceipt) -> SyncResult<()> {
        sqlx::query(
            r#"
            INSERT INTO sync.sync_batch_log
                (sync_id, source_node, target_node, partition_key, direction,
                 events_count, bytes_transferred, conflict_count, conflicts_auto,
                 conflicts_manual, local_merkle, remote_merkle, source_sig,
                 target_sig, compression_ratio, duration_ms, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            "#,
        )
        .bind(receipt.sync_id)
        .bind(receipt.source_node)
        .bind(receipt.target_node)
        .bind(&receipt.partition_key)
        .bind(match receipt.direction {
            SyncDirection::Upload => "upload",
            SyncDirection::Download => "download",
            SyncDirection::Bidirectional => "bidirectional",
        })
        .bind(receipt.events_count as i32)
        .bind(receipt.bytes_transferred as i64)
        .bind(receipt.conflict_count as i32)
        .bind(receipt.conflicts_auto as i32)
        .bind(receipt.conflicts_manual as i32)
        .bind(&receipt.local_merkle)
        .bind(&receipt.remote_merkle)
        .bind(&receipt.source_signature)
        .bind(&receipt.target_signature)
        .bind(receipt.compression_ratio)
        .bind(receipt.duration_ms as i64)
        .bind(receipt.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_sync_id(&self, sync_id: uuid::Uuid) -> SyncResult<Option<BatchReceipt>> {
        let row = sqlx::query_as::<_, BatchLogRow>(
            "SELECT * FROM sync.sync_batch_log WHERE sync_id = $1",
        )
        .bind(sync_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(TryInto::try_into).transpose()
    }

    pub async fn list_recent(&self, limit: i64) -> SyncResult<Vec<BatchReceipt>> {
        let rows = sqlx::query_as::<_, BatchLogRow>(
            "SELECT * FROM sync.sync_batch_log ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| r.try_into()).collect()
    }

    pub async fn count_by_partition(&self, partition_key: &str) -> SyncResult<i64> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM sync.sync_batch_log WHERE partition_key = $1")
                .bind(partition_key)
                .fetch_one(&self.pool)
                .await?;
        Ok(row.0)
    }
}

#[derive(sqlx::FromRow)]
struct BatchLogRow {
    sync_id: uuid::Uuid,
    source_node: uuid::Uuid,
    target_node: uuid::Uuid,
    partition_key: String,
    direction: String,
    events_count: i32,
    bytes_transferred: i64,
    conflict_count: i32,
    conflicts_auto: i32,
    conflicts_manual: i32,
    local_merkle: Vec<u8>,
    remote_merkle: Vec<u8>,
    source_sig: Vec<u8>,
    target_sig: Vec<u8>,
    compression_ratio: f64,
    duration_ms: i64,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl TryFrom<BatchLogRow> for BatchReceipt {
    type Error = crate::error::SyncEngineError;

    fn try_from(row: BatchLogRow) -> Result<Self, Self::Error> {
        let direction = match row.direction.as_str() {
            "upload" => SyncDirection::Upload,
            "download" => SyncDirection::Download,
            "bidirectional" => SyncDirection::Bidirectional,
            _ => {
                return Err(Self::Error::Internal(format!(
                    "Unknown direction: {}",
                    row.direction
                )))
            }
        };

        Ok(BatchReceipt {
            sync_id: row.sync_id,
            source_node: row.source_node,
            target_node: row.target_node,
            partition_key: row.partition_key,
            direction,
            events_count: row.events_count as u32,
            bytes_transferred: row.bytes_transferred as u64,
            conflict_count: row.conflict_count as u32,
            conflicts_auto: row.conflicts_auto as u32,
            conflicts_manual: row.conflicts_manual as u32,
            local_merkle: row.local_merkle,
            remote_merkle: row.remote_merkle,
            source_signature: row.source_sig,
            target_signature: row.target_sig,
            compression_ratio: row.compression_ratio,
            duration_ms: row.duration_ms as u64,
            created_at: row.created_at,
        })
    }
}
