use crate::error::{SyncEngineError, SyncResult};
use sqlx::PgPool;

pub async fn run_migrations(pool: &PgPool) -> SyncResult<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| SyncEngineError::Storage(e.into()))?;
    Ok(())
}
