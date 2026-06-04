use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineEntry {
    pub conflict_id: uuid::Uuid,
    pub partition_key: String,
    pub record_id: String,
    pub record_type: String,
    pub local_payload: Vec<u8>,
    pub remote_payload: Vec<u8>,
    pub strategy: String,
    pub status: QuarantineStatus,
    pub escalated_at: chrono::DateTime<chrono::Utc>,
    pub resolved_by: Option<uuid::Uuid>,
    pub resolution: Option<String>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuarantineStatus {
    Open,
    AutoResolved,
    ManualResolved,
    Escalated,
    Expired,
}

pub struct QuarantineManager {
    max_quarantine_days: u32,
}

impl QuarantineManager {
    pub fn new(max_quarantine_days: u32) -> Self {
        Self {
            max_quarantine_days,
        }
    }

    pub fn is_expired(&self, entry: &QuarantineEntry) -> bool {
        let age = chrono::Utc::now() - entry.created_at;
        age.num_days() > self.max_quarantine_days as i64
    }

    pub fn should_escalate(&self, entry: &QuarantineEntry) -> bool {
        if !matches!(entry.status, QuarantineStatus::Open) {
            return false;
        }
        let age = chrono::Utc::now() - entry.created_at;
        age.num_days() > (self.max_quarantine_days / 2) as i64
    }
}
