pub mod batch_log_repo;
pub mod checkpoint_repo;
pub mod conflict_repo;
pub mod event_store;
pub mod migration;
pub mod node_repo;
pub mod queue_repo;

pub use batch_log_repo::*;
pub use checkpoint_repo::*;
pub use conflict_repo::*;
pub use event_store::*;
pub use migration::*;
pub use node_repo::*;
pub use queue_repo::*;

use crate::config::StorageConfig;
use crate::error::SyncResult;
use sqlx::postgres::{PgPool, PgPoolOptions};

pub type Pool = PgPool;

pub async fn create_pool(config: &StorageConfig) -> SyncResult<Pool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_secs))
        .max_lifetime(std::time::Duration::from_secs(config.max_lifetime_secs))
        .idle_timeout(std::time::Duration::from_secs(config.idle_timeout_secs))
        .connect(&config.database_url)
        .await?;

    Ok(pool)
}

pub struct PgStore {
    pub pool: Pool,
    pub nodes: NodeRepo,
    pub checkpoints: CheckpointRepo,
    pub batch_log: BatchLogRepo,
    pub conflicts: ConflictRepo,
    pub queue: QueueRepo,
    pub events: EventStore,
}

impl PgStore {
    pub async fn new(config: &StorageConfig) -> SyncResult<Self> {
        let pool = create_pool(config).await?;

        if config.run_migrations {
            migration::run_migrations(&pool).await?;
        }

        Ok(Self {
            pool: pool.clone(),
            nodes: NodeRepo::new(pool.clone()),
            checkpoints: CheckpointRepo::new(pool.clone()),
            batch_log: BatchLogRepo::new(pool.clone()),
            conflicts: ConflictRepo::new(pool.clone()),
            queue: QueueRepo::new(pool.clone()),
            events: EventStore::new(pool.clone()),
        })
    }
}
