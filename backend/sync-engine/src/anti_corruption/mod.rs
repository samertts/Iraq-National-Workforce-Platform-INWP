pub mod anomaly;
pub mod quarantine;
pub mod scanner;

pub use anomaly::*;
pub use quarantine::*;
pub use scanner::*;

use serde::{Deserialize, Serialize};

/// Severity classification for corruption events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorruptionSeverity {
    Info,
    Suspicious,
    Minor,
    Major,
    Critical,
    Catastrophic,
}

/// Classification of detected anomaly types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    InvalidSignature,
    ReplayAttack,
    ForkedHistory,
    OutOfOrderEvent,
    DuplicateEvent,
    MissingAncestor,
    SchemaViolation,
    TamperedPayload,
    UnauthorizedMutation,
    TimeTravel,
    ConflictingTopology,
    RogueNodeActivity,
}

impl AnomalyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidSignature => "invalid_signature",
            Self::ReplayAttack => "replay_attack",
            Self::ForkedHistory => "forked_history",
            Self::OutOfOrderEvent => "out_of_order_event",
            Self::DuplicateEvent => "duplicate_event",
            Self::MissingAncestor => "missing_ancestor",
            Self::SchemaViolation => "schema_violation",
            Self::TamperedPayload => "tampered_payload",
            Self::UnauthorizedMutation => "unauthorized_mutation",
            Self::TimeTravel => "time_travel",
            Self::ConflictingTopology => "conflicting_topology",
            Self::RogueNodeActivity => "rogue_node_activity",
        }
    }
}

/// A detected corruption event with full forensic context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorruptionEvent {
    pub event_id: uuid::Uuid,
    pub anomaly_type: AnomalyType,
    pub severity: CorruptionSeverity,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub source_node: uuid::Uuid,
    pub affected_records: Vec<String>,
    pub description: String,
    pub evidence: Vec<u8>,
    pub quarantined: bool,
    pub resolved: bool,
}

impl CorruptionEvent {
    pub fn new(
        anomaly_type: AnomalyType,
        severity: CorruptionSeverity,
        source_node: uuid::Uuid,
        description: impl Into<String>,
        evidence: Vec<u8>,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::now_v7(),
            anomaly_type,
            severity,
            detected_at: chrono::Utc::now(),
            source_node,
            affected_records: Vec::new(),
            description: description.into(),
            evidence,
            quarantined: false,
            resolved: false,
        }
    }
}
