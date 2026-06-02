use crate::core::Checkpoint;
use crate::error::SyncResult;
use sqlx::PgPool;

#[derive(Clone)]
pub struct CheckpointRepo {
    pool: PgPool,
}

impl CheckpointRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, checkpoint: &Checkpoint) -> SyncResult<()> {
        sqlx::query(
            r#"
            INSERT INTO sync.sync_checkpoint
                (node_id, partition_key, merkle_root, last_sync_at, synced_events, last_error)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (node_id, partition_key) DO UPDATE SET
                merkle_root = EXCLUDED.merkle_root,
                last_sync_at = EXCLUDED.last_sync_at,
                synced_events = EXCLUDED.synced_events,
                last_error = EXCLUDED.last_error
            "#,
        )
        .bind(checkpoint.node_id)
        .bind(&checkpoint.partition_key)
        .bind(&checkpoint.merkle_root)
        .bind(checkpoint.last_sync_at)
        .bind(checkpoint.synced_events as i64)
        .bind(&checkpoint.last_error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_partition(
        &self,
        node_id: uuid::Uuid,
        partition_key: &str,
    ) -> SyncResult<Option<Checkpoint>> {
        let row = sqlx::query_as::<_, CheckpointRow>(
            "SELECT * FROM sync.sync_checkpoint WHERE node_id = $1 AND partition_key = $2",
        )
        .bind(node_id)
        .bind(partition_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Checkpoint {
            node_id: r.node_id,
            partition_key: r.partition_key,
            merkle_root: r.merkle_root,
            last_sync_at: r.last_sync_at,
            synced_events: r.synced_events as u64,
            last_error: r.last_error,
        }))
    }

    pub async fn list_for_node(&self, node_id: uuid::Uuid) -> SyncResult<Vec<Checkpoint>> {
        let rows = sqlx::query_as::<_, CheckpointRow>(
            "SELECT * FROM sync.sync_checkpoint WHERE node_id = $1 ORDER BY partition_key",
        )
        .bind(node_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| Checkpoint {
                node_id: r.node_id,
                partition_key: r.partition_key,
                merkle_root: r.merkle_root,
                last_sync_at: r.last_sync_at,
                synced_events: r.synced_events as u64,
                last_error: r.last_error,
            })
            .collect())
    }

    pub async fn list_all(&self) -> SyncResult<Vec<Checkpoint>> {
        let rows = sqlx::query_as::<_, CheckpointRow>("SELECT * FROM sync.sync_checkpoint")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|r| Checkpoint {
                node_id: r.node_id,
                partition_key: r.partition_key,
                merkle_root: r.merkle_root,
                last_sync_at: r.last_sync_at,
                synced_events: r.synced_events as u64,
                last_error: r.last_error,
            })
            .collect())
    }
}

#[derive(sqlx::FromRow)]
struct CheckpointRow {
    node_id: uuid::Uuid,
    partition_key: String,
    merkle_root: Vec<u8>,
    last_sync_at: chrono::DateTime<chrono::Utc>,
    synced_events: i64,
    last_error: Option<String>,
}
