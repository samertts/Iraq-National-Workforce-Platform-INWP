use serde::{Deserialize, Serialize};

pub type RecordId = String;
pub type NodeId = uuid::Uuid;
pub type SyncSessionId = uuid::Uuid;
pub type EventId = uuid::Uuid;
pub type ConflictId = uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncPhase {
    Discovery,
    MerkleExchange,
    DeltaTransfer,
    Reconciliation,
    Commitment,
    Completed,
    Failed,
}

impl SyncPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Discovery => "discovery",
            Self::MerkleExchange => "merkle_exchange",
            Self::DeltaTransfer => "delta_transfer",
            Self::Reconciliation => "reconciliation",
            Self::Commitment => "commitment",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncMode {
    Full,
    Delta,
    Replay,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordOperation {
    Create,
    Update,
    Delete,
    Tombstone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    pub record_id: RecordId,
    pub record_type: String,
    pub payload: Vec<u8>,
    pub version_vector: crate::core::version::VersionVector,
    pub local_timestamp: chrono::DateTime<chrono::Utc>,
    pub signature: Vec<u8>,
    pub operation: RecordOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncBatch {
    pub sync_id: SyncSessionId,
    pub partition_key: String,
    pub direction: SyncDirection,
    pub records: Vec<SyncRecord>,
    pub batch_seq: u32,
    pub is_final: bool,
    pub merkle_proof: Vec<u8>,
    pub compression: Option<CompressionInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncDirection {
    Upload,
    Download,
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    pub algorithm: String,
    pub level: i32,
    pub original_size: u64,
    pub compressed_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReceipt {
    pub sync_id: SyncSessionId,
    pub source_node: NodeId,
    pub target_node: NodeId,
    pub partition_key: String,
    pub direction: SyncDirection,
    pub events_count: u32,
    pub bytes_transferred: u64,
    pub conflict_count: u32,
    pub conflicts_auto: u32,
    pub conflicts_manual: u32,
    pub local_merkle: Vec<u8>,
    pub remote_merkle: Vec<u8>,
    pub source_signature: Vec<u8>,
    pub target_signature: Vec<u8>,
    pub compression_ratio: f64,
    pub duration_ms: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncEventType {
    BatchCommitted,
    ConflictDetected,
    ConflictResolved,
    HeartbeatSent,
    SchemaNegotiated,
    NodeStateChanged,
    PeerDiscovered,
    PeerLost,
    SyncStarted,
    SyncCompleted,
    SyncFailed,
    CheckpointAdvanced,
    CorruptionDetected,
    RecoveryStarted,
    RecoveryCompleted,
    ReplayStarted,
    ReplayCompleted,
}

impl SyncEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BatchCommitted => "inwp.sync.v1.batch.committed",
            Self::ConflictDetected => "inwp.sync.v1.conflict.detected",
            Self::ConflictResolved => "inwp.sync.v1.conflict.resolved",
            Self::HeartbeatSent => "inwp.sync.v1.heartbeat.sent",
            Self::SchemaNegotiated => "inwp.sync.v1.schema.negotiated",
            Self::NodeStateChanged => "inwp.sync.v1.node.state.changed",
            Self::PeerDiscovered => "inwp.sync.v1.peer.discovered",
            Self::PeerLost => "inwp.sync.v1.peer.lost",
            Self::SyncStarted => "inwp.sync.v1.sync.started",
            Self::SyncCompleted => "inwp.sync.v1.sync.completed",
            Self::SyncFailed => "inwp.sync.v1.sync.failed",
            Self::CheckpointAdvanced => "inwp.sync.v1.checkpoint.advanced",
            Self::CorruptionDetected => "inwp.sync.v1.corruption.detected",
            Self::RecoveryStarted => "inwp.sync.v1.recovery.started",
            Self::RecoveryCompleted => "inwp.sync.v1.recovery.completed",
            Self::ReplayStarted => "inwp.sync.v1.replay.started",
            Self::ReplayCompleted => "inwp.sync.v1.replay.completed",
        }
    }
}
