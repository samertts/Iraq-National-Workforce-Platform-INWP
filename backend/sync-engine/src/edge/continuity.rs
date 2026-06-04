use sha2::{Digest, Sha256};

use super::EdgeContinuityRecord;

pub struct EdgeContinuityManager {
    continuity_log: Vec<EdgeContinuityRecord>,
    last_continuity_token: uuid::Uuid,
}

impl Default for EdgeContinuityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EdgeContinuityManager {
    pub fn new() -> Self {
        Self {
            continuity_log: Vec::new(),
            last_continuity_token: uuid::Uuid::now_v7(),
        }
    }

    pub fn record_operation(
        &mut self,
        record_id: impl Into<String>,
        operation: super::EdgeOperation,
        payload: &[u8],
        node_id: uuid::Uuid,
    ) -> EdgeContinuityRecord {
        let record = EdgeContinuityRecord {
            record_id: record_id.into(),
            operation,
            local_timestamp: chrono::Utc::now(),
            payload_hash: Sha256::new().chain_update(payload).finalize().to_vec(),
            version: self.continuity_log.len() as u64 + 1,
            signed_by: node_id,
        };
        self.continuity_log.push(record.clone());
        self.last_continuity_token = uuid::Uuid::now_v7();
        record
    }

    pub fn replay_since(&self, token: uuid::Uuid) -> Vec<&EdgeContinuityRecord> {
        let start_idx = self
            .continuity_log
            .iter()
            .position(|r| r.payload_hash.starts_with(token.as_bytes()))
            .unwrap_or(0);
        self.continuity_log[start_idx..].iter().collect()
    }

    pub fn continuity_log(&self) -> &[EdgeContinuityRecord] {
        &self.continuity_log
    }

    pub fn last_token(&self) -> uuid::Uuid {
        self.last_continuity_token
    }

    pub fn clear_confirmed(&mut self, up_to_version: u64) {
        self.continuity_log.retain(|r| r.version > up_to_version);
    }
}
