pub mod health;
pub mod metrics;

pub use health::*;
pub use metrics::*;

use crate::config::ObservabilityConfig;
use crate::error::SyncResult;

pub struct ObservabilityGuard;

impl Drop for ObservabilityGuard {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}

pub fn init_observability(config: &ObservabilityConfig) -> SyncResult<ObservabilityGuard> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log_level));

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| crate::error::SyncEngineError::Config(format!("Tracing setup failed: {}", e)))?;

    Ok(ObservabilityGuard)
}
