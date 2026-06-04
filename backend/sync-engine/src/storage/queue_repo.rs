use crate::error::SyncResult;
use sqlx::PgPool;

#[derive(Clone)]
pub struct QueueRepo {
    pool: PgPool,
}

impl QueueRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn enqueue(
        &self,
        node_id: uuid::Uuid,
        partition_key: &str,
        event_id: uuid::Uuid,
        event_type: &str,
        payload: &[u8],
        priority: i32,
    ) -> SyncResult<()> {
        sqlx::query(
            r#"
            INSERT INTO sync.sync_queue
                (node_id, partition_key, event_id, event_type, payload, priority, status)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending')
            "#,
        )
        .bind(node_id)
        .bind(partition_key)
        .bind(event_id)
        .bind(event_type)
        .bind(payload)
        .bind(priority)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn dequeue(&self, node_id: uuid::Uuid, limit: i64) -> SyncResult<Vec<QueueItem>> {
        let rows = sqlx::query_as::<_, QueueRow>(
            r#"
            UPDATE sync.sync_queue
            SET status = 'processing'
            WHERE queue_id IN (
                SELECT queue_id FROM sync.sync_queue
                WHERE node_id = $1 AND status = 'pending'
                ORDER BY priority ASC, created_at ASC
                LIMIT $2
                FOR UPDATE SKIP LOCKED
            )
            RETURNING *
            "#,
        )
        .bind(node_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(QueueItem::from).collect())
    }

    pub async fn mark_failed(&self, queue_id: uuid::Uuid, error: &str) -> SyncResult<()> {
        sqlx::query(
            r#"
            UPDATE sync.sync_queue
            SET status = 'failed', last_error = $1, retry_count = retry_count + 1
            WHERE queue_id = $2
            "#,
        )
        .bind(error)
        .bind(queue_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_completed(&self, queue_id: uuid::Uuid) -> SyncResult<()> {
        sqlx::query("UPDATE sync.sync_queue SET status = 'completed' WHERE queue_id = $1")
            .bind(queue_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn retry_failed(&self, max_retries: i32) -> SyncResult<u64> {
        let result = sqlx::query(
            r#"
            UPDATE sync.sync_queue
            SET status = 'pending', next_retry_at = $1, last_error = NULL
            WHERE status = 'failed' AND retry_count < $2
            "#,
        )
        .bind(chrono::Utc::now())
        .bind(max_retries)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn depth(&self, node_id: uuid::Uuid) -> SyncResult<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sync.sync_queue WHERE node_id = $1 AND status = 'pending'",
        )
        .bind(node_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.0)
    }

    pub async fn purge_completed(&self, before: chrono::DateTime<chrono::Utc>) -> SyncResult<u64> {
        let result = sqlx::query(
            "DELETE FROM sync.sync_queue WHERE status = 'completed' AND created_at < $1",
        )
        .bind(before)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

#[derive(Debug)]
pub struct QueueItem {
    pub queue_id: uuid::Uuid,
    pub node_id: uuid::Uuid,
    pub partition_key: String,
    pub event_id: uuid::Uuid,
    pub event_type: String,
    pub payload: Vec<u8>,
    pub priority: i32,
    pub status: String,
    pub retry_count: i32,
    pub last_error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow)]
struct QueueRow {
    queue_id: uuid::Uuid,
    node_id: uuid::Uuid,
    partition_key: String,
    event_id: uuid::Uuid,
    event_type: String,
    payload: Vec<u8>,
    priority: i32,
    status: String,
    retry_count: i32,
    last_error: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    next_retry_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<QueueRow> for QueueItem {
    fn from(row: QueueRow) -> Self {
        Self {
            queue_id: row.queue_id,
            node_id: row.node_id,
            partition_key: row.partition_key,
            event_id: row.event_id,
            event_type: row.event_type,
            payload: row.payload,
            priority: row.priority,
            status: row.status,
            retry_count: row.retry_count,
            last_error: row.last_error,
            created_at: row.created_at,
            next_retry_at: row.next_retry_at,
        }
    }
}
