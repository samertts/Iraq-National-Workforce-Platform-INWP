pub mod autonomy;
pub mod cache;
pub mod continuity;

pub use autonomy::*;
pub use cache::*;
pub use continuity::*;

use serde::{Deserialize, Serialize};

/// Edge isolation state — the complete operational posture of a disconnected node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeIsolationState {
    pub node_id: uuid::Uuid,
    pub domain_id: uuid::Uuid,
    pub autonomous_mode: bool,
    pub disconnected_since: Option<chrono::DateTime<chrono::Utc>>,
    pub local_queue_depth: u64,
    pub last_successful_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub corruption_quarantine_count: u32,
    pub pending_reconciliation_count: u32,
    pub local_continuity_token: uuid::Uuid,
    pub edge_capabilities: EdgeCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeCapabilities {
    pub max_offline_duration_days: u32,
    pub local_auth_enabled: bool,
    pub local_workflow_enabled: bool,
    pub max_pending_events: u64,
    pub storage_capacity_bytes: u64,
    pub supported_entities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeContinuityRecord {
    pub record_id: String,
    pub operation: EdgeOperation,
    pub local_timestamp: chrono::DateTime<chrono::Utc>,
    pub payload_hash: Vec<u8>,
    pub version: u64,
    pub signed_by: uuid::Uuid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeOperation {
    Create,
    Update,
    Delete,
    Approve,
    Reject,
    Escalate,
}
