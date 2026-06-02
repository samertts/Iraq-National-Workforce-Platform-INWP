use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEvent {
    pub event_id: uuid::Uuid,
    pub node_id: uuid::Uuid,
    pub event_type: String,
    pub partition_key: String,
    pub payload: Vec<u8>,
    pub version_vector: crate::core::VersionVector,
    pub local_timestamp: u64,
    pub signature: Vec<u8>,
    pub signing_key_id: String,
    pub schema_version: String,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SyncEvent {
    pub fn new(
        node_id: uuid::Uuid,
        event_type: impl Into<String>,
        partition_key: impl Into<String>,
        payload: Vec<u8>,
        signing_key_id: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            event_id: uuid::Uuid::now_v7(),
            node_id,
            event_type: event_type.into(),
            partition_key: partition_key.into(),
            payload,
            version_vector: crate::core::VersionVector::new(node_id),
            local_timestamp: now.timestamp_nanos_opt().unwrap_or(0) as u64,
            signature: Vec::new(),
            signing_key_id: signing_key_id.into(),
            schema_version: "1.0".into(),
            metadata: HashMap::new(),
            created_at: now,
        }
    }

    pub fn with_version_vector(mut self, vv: crate::core::VersionVector) -> Self {
        self.version_vector = vv;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn event_type_str(&self) -> &str {
        &self.event_type
    }

    pub fn is_type(&self, event_type: &str) -> bool {
        self.event_type == event_type
    }
}

pub mod types {
    pub const BATCH_COMMITTED: &str = "inwp.sync.v1.batch.committed";
    pub const CONFLICT_DETECTED: &str = "inwp.sync.v1.conflict.detected";
    pub const CONFLICT_RESOLVED: &str = "inwp.sync.v1.conflict.resolved";
    pub const HEARTBEAT_SENT: &str = "inwp.sync.v1.heartbeat.sent";
    pub const SCHEMA_NEGOTIATED: &str = "inwp.sync.v1.schema.negotiated";
    pub const NODE_STATE_CHANGED: &str = "inwp.sync.v1.node.state.changed";
    pub const PEER_DISCOVERED: &str = "inwp.sync.v1.peer.discovered";
    pub const PEER_LOST: &str = "inwp.sync.v1.peer.lost";
    pub const SYNC_STARTED: &str = "inwp.sync.v1.sync.started";
    pub const SYNC_COMPLETED: &str = "inwp.sync.v1.sync.completed";
    pub const SYNC_FAILED: &str = "inwp.sync.v1.sync.failed";
    pub const CHECKPOINT_ADVANCED: &str = "inwp.sync.v1.checkpoint.advanced";
    pub const CORRUPTION_DETECTED: &str = "inwp.sync.v1.corruption.detected";
    pub const RECOVERY_STARTED: &str = "inwp.sync.v1.recovery.started";
    pub const RECOVERY_COMPLETED: &str = "inwp.sync.v1.recovery.completed";
    pub const REPLAY_STARTED: &str = "inwp.sync.v1.replay.started";
    pub const REPLAY_COMPLETED: &str = "inwp.sync.v1.replay.completed";
}
