use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEngineConfig {
    pub node: NodeConfig,
    pub storage: StorageConfig,
    pub transport: TransportConfig,
    pub protocol: ProtocolConfig,
    pub security: SecurityConfig,
    pub observability: ObservabilityConfig,
    pub recovery: RecoveryConfig,
    pub reconciliation: ReconciliationConfig,
    pub events: EventConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_id: String,
    pub node_name: String,
    pub node_type: NodeTypeConfig,
    pub ministry_id: String,
    pub site_id: String,
    pub region: String,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeTypeConfig {
    NationalHub,
    RegionalRelay,
    Edge,
    Mobile,
    DrReplica,
}

impl NodeTypeConfig {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NationalHub => "national_hub",
            Self::RegionalRelay => "regional_relay",
            Self::Edge => "edge",
            Self::Mobile => "mobile",
            Self::DrReplica => "dr_replica",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_secs: u64,
    pub max_lifetime_secs: u64,
    pub idle_timeout_secs: u64,
    pub run_migrations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub grpc_listen: String,
    pub grpc_port: u16,
    pub mesh_port: u16,
    pub http_port: u16,
    pub websocket_port: u16,
    pub nats_url: Option<String>,
    pub mdns_enabled: bool,
    pub mdns_service_name: String,
    pub max_message_size: usize,
    pub keepalive_interval_secs: u64,
    pub keepalive_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub max_batch_size: u32,
    pub max_batch_bytes: u64,
    pub compression_level_full: i32,
    pub compression_level_delta: i32,
    pub chunk_size_bytes: u64,
    pub bloom_filter_fp_rate: f64,
    pub sync_timeout_secs: u64,
    pub max_concurrent_syncs: u32,
    pub heartbeat_interval_secs: u64,
    pub heartbeat_missed_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub tls_cert_path: String,
    pub tls_key_path: String,
    pub tls_ca_path: String,
    pub signing_key_path: String,
    pub signing_key_id: String,
    pub mtls_required: bool,
    pub cert_verify_crl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub metrics_port: u16,
    pub tracing_endpoint: Option<String>,
    pub tracing_service_name: String,
    pub log_level: String,
    pub log_format: String,
    pub health_check_port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    pub max_retries: u32,
    pub base_retry_delay_ms: u64,
    pub max_retry_delay_ms: u64,
    pub jitter_ms: u64,
    pub checkpoint_interval_events: u64,
    pub checkpoint_interval_secs: u64,
    pub replay_batch_size: u32,
    pub replay_verify_merkle: bool,
    pub quarantine_timeout_days: u32,
    pub corruption_check_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconciliationConfig {
    pub default_strategy: String,
    pub auto_resolve: bool,
    pub escalation_timeout_hours: u32,
    pub max_manual_conflicts: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    pub event_retention_days: u32,
    pub max_event_size_bytes: u64,
    pub event_bus_type: String,
    pub publish_own_events: bool,
}

impl SyncEngineConfig {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, config::ConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            config::ConfigError::Message(format!("Failed to read config file: {}", e))
        })?;
        toml::from_str(&content)
            .map_err(|e| config::ConfigError::Message(format!("Failed to parse config: {}", e)))
    }

    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();

        Ok(Self {
            node: NodeConfig {
                node_id: std::env::var("SYNC_NODE_ID").unwrap_or_else(|_| "node-001".into()),
                node_name: std::env::var("SYNC_NODE_NAME").unwrap_or_else(|_| "node-001".into()),
                node_type: match std::env::var("SYNC_NODE_TYPE").as_deref() {
                    Ok("national_hub") => NodeTypeConfig::NationalHub,
                    Ok("regional_relay") => NodeTypeConfig::RegionalRelay,
                    Ok("edge") => NodeTypeConfig::Edge,
                    Ok("mobile") => NodeTypeConfig::Mobile,
                    Ok("dr_replica") => NodeTypeConfig::DrReplica,
                    _ => NodeTypeConfig::Edge,
                },
                ministry_id: std::env::var("SYNC_MINISTRY_ID")
                    .unwrap_or_else(|_| "ministry-default".into()),
                site_id: std::env::var("SYNC_SITE_ID").unwrap_or_else(|_| "site-default".into()),
                region: std::env::var("SYNC_REGION").unwrap_or_else(|_| "region-default".into()),
                data_dir: std::env::var("SYNC_DATA_DIR")
                    .unwrap_or_else(|_| "/data/sync-engine".into()),
            },
            storage: StorageConfig {
                database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
                max_connections: std::env::var("DB_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(20),
                min_connections: std::env::var("DB_MIN_CONNECTIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5),
                acquire_timeout_secs: std::env::var("DB_ACQUIRE_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30),
                max_lifetime_secs: std::env::var("DB_MAX_LIFETIME")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1800),
                idle_timeout_secs: std::env::var("DB_IDLE_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(600),
                run_migrations: std::env::var("DB_RUN_MIGRATIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
            },
            transport: TransportConfig {
                grpc_listen: std::env::var("SYNC_GRPC_LISTEN").unwrap_or_else(|_| "0.0.0.0".into()),
                grpc_port: std::env::var("SYNC_GRPC_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(50052),
                mesh_port: std::env::var("SYNC_MESH_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(50053),
                http_port: std::env::var("SYNC_HTTP_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(8080),
                websocket_port: std::env::var("SYNC_WS_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(8081),
                nats_url: std::env::var("NATS_URL").ok(),
                mdns_enabled: std::env::var("SYNC_MDNS_ENABLED")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
                mdns_service_name: std::env::var("SYNC_MDNS_SERVICE")
                    .unwrap_or_else(|_| "_inwp-sync._tcp.local.".into()),
                max_message_size: std::env::var("SYNC_MAX_MSG_SIZE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10 * 1024 * 1024),
                keepalive_interval_secs: std::env::var("SYNC_KEEPALIVE_INTERVAL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30),
                keepalive_timeout_secs: std::env::var("SYNC_KEEPALIVE_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
            },
            protocol: ProtocolConfig {
                max_batch_size: std::env::var("SYNC_MAX_BATCH_SIZE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1000),
                max_batch_bytes: std::env::var("SYNC_MAX_BATCH_BYTES")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1024 * 1024),
                compression_level_full: std::env::var("SYNC_COMPRESSION_FULL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(19),
                compression_level_delta: std::env::var("SYNC_COMPRESSION_DELTA")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(3),
                chunk_size_bytes: std::env::var("SYNC_CHUNK_SIZE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1024 * 1024),
                bloom_filter_fp_rate: std::env::var("SYNC_BLOOM_FP_RATE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0.01),
                sync_timeout_secs: std::env::var("SYNC_TIMEOUT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(300),
                max_concurrent_syncs: std::env::var("SYNC_MAX_CONCURRENT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
                heartbeat_interval_secs: std::env::var("SYNC_HEARTBEAT_INTERVAL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(60),
                heartbeat_missed_threshold: std::env::var("SYNC_HEARTBEAT_MISSED")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(3),
            },
            security: SecurityConfig {
                tls_cert_path: std::env::var("SYNC_TLS_CERT").expect("SYNC_TLS_CERT must be set"),
                tls_key_path: std::env::var("SYNC_TLS_KEY").expect("SYNC_TLS_KEY must be set"),
                tls_ca_path: std::env::var("SYNC_TLS_CA").expect("SYNC_TLS_CA must be set"),
                signing_key_path: std::env::var("SYNC_SIGNING_KEY")
                    .expect("SYNC_SIGNING_KEY must be set"),
                signing_key_id: std::env::var("SYNC_SIGNING_KEY_ID")
                    .unwrap_or_else(|_| "default-key".into()),
                mtls_required: std::env::var("SYNC_MTLS_REQUIRED")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
                cert_verify_crl: std::env::var("SYNC_CERT_VERIFY_CRL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
            },
            observability: ObservabilityConfig {
                metrics_port: std::env::var("SYNC_METRICS_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(9090),
                tracing_endpoint: std::env::var("OTLP_ENDPOINT").ok(),
                tracing_service_name: std::env::var("OTLP_SERVICE_NAME")
                    .unwrap_or_else(|_| "sync-engine".into()),
                log_level: std::env::var("SYNC_LOG_LEVEL").unwrap_or_else(|_| "info".into()),
                log_format: std::env::var("SYNC_LOG_FORMAT").unwrap_or_else(|_| "json".into()),
                health_check_port: std::env::var("SYNC_HEALTH_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(8082),
            },
            recovery: RecoveryConfig {
                max_retries: std::env::var("SYNC_MAX_RETRIES")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
                base_retry_delay_ms: std::env::var("SYNC_BASE_RETRY_MS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1000),
                max_retry_delay_ms: std::env::var("SYNC_MAX_RETRY_MS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1_800_000),
                jitter_ms: std::env::var("SYNC_JITTER_MS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1000),
                checkpoint_interval_events: std::env::var("SYNC_CHECKPOINT_EVENTS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10_000),
                checkpoint_interval_secs: std::env::var("SYNC_CHECKPOINT_SECS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(3600),
                replay_batch_size: std::env::var("SYNC_REPLAY_BATCH")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1000),
                replay_verify_merkle: std::env::var("SYNC_REPLAY_VERIFY")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
                quarantine_timeout_days: std::env::var("SYNC_QUARANTINE_DAYS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(30),
                corruption_check_interval_secs: std::env::var("SYNC_CORRUPTION_CHECK")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(86_400),
            },
            reconciliation: ReconciliationConfig {
                default_strategy: std::env::var("SYNC_DEFAULT_STRATEGY")
                    .unwrap_or_else(|_| "lww".into()),
                auto_resolve: std::env::var("SYNC_AUTO_RESOLVE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
                escalation_timeout_hours: std::env::var("SYNC_ESCALATION_HOURS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(72),
                max_manual_conflicts: std::env::var("SYNC_MAX_MANUAL")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1000),
            },
            events: EventConfig {
                event_retention_days: std::env::var("SYNC_EVENT_RETENTION")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(365),
                max_event_size_bytes: std::env::var("SYNC_MAX_EVENT_SIZE")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(1024 * 1024),
                event_bus_type: std::env::var("SYNC_EVENT_BUS").unwrap_or_else(|_| "nats".into()),
                publish_own_events: std::env::var("SYNC_PUBLISH_OWN")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
            },
        })
    }

    pub fn node_id_bytes(&self) -> [u8; 16] {
        let id = uuid::Uuid::parse_str(&self.node.node_id).unwrap_or_else(|_| uuid::Uuid::nil());
        *id.as_bytes()
    }
}

#[allow(clippy::module_inception)]
mod config {
    #[derive(Debug)]
    pub enum ConfigError {
        Message(String),
    }

    impl std::fmt::Display for ConfigError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Message(msg) => write!(f, "{}", msg),
            }
        }
    }

    impl std::error::Error for ConfigError {}
}
