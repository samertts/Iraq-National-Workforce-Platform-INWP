use crate::error::SyncResult;

pub async fn health_check() -> SyncResult<HealthStatus> {
    Ok(HealthStatus {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        uptime: chrono::Utc::now().to_rfc3339(),
    })
}

#[derive(Debug, serde::Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime: String,
}

pub async fn readiness_check(pool: &sqlx::PgPool) -> SyncResult<ReadinessStatus> {
    let db_ok = sqlx::query("SELECT 1").execute(pool).await.is_ok();

    Ok(ReadinessStatus {
        ready: db_ok,
        database: if db_ok {
            "connected".to_string()
        } else {
            "disconnected".to_string()
        },
    })
}

#[derive(Debug, serde::Serialize)]
pub struct ReadinessStatus {
    pub ready: bool,
    pub database: String,
}
