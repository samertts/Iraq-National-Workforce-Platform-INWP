use crate::error::SyncResult;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Cryptographically chained event — each event links to the previous via hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainedEvent {
    pub event_id: uuid::Uuid,
    pub previous_hash: Vec<u8>,
    pub event_hash: Vec<u8>,
    pub payload: Vec<u8>,
    pub node_id: uuid::Uuid,
    pub domain_id: uuid::Uuid,
    pub chain_depth: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub signature: Vec<u8>,
}

impl ChainedEvent {
    pub fn new(
        previous_hash: Vec<u8>,
        payload: Vec<u8>,
        node_id: uuid::Uuid,
        domain_id: uuid::Uuid,
        chain_depth: u64,
    ) -> Self {
        let event_id = uuid::Uuid::now_v7();
        let timestamp = chrono::Utc::now();

        let mut hasher = Sha256::new();
        hasher.update(&previous_hash);
        hasher.update(&payload);
        hasher.update(node_id.as_bytes());
        hasher.update(domain_id.as_bytes());
        hasher.update(chain_depth.to_le_bytes());
        hasher.update(timestamp.to_rfc3339().as_bytes());
        let event_hash = hasher.finalize().to_vec();

        Self {
            event_id,
            previous_hash,
            event_hash,
            payload,
            node_id,
            domain_id,
            chain_depth,
            timestamp,
            signature: Vec::new(),
        }
    }

    pub fn verify_chain_integrity(&self) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(&self.previous_hash);
        hasher.update(&self.payload);
        hasher.update(self.node_id.as_bytes());
        hasher.update(self.domain_id.as_bytes());
        hasher.update(self.chain_depth.to_le_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        let computed = hasher.finalize().to_vec();
        computed == self.event_hash
    }
}

/// Immutable chain of events — the backbone of the sovereign event store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventChain {
    pub domain_id: uuid::Uuid,
    pub partition_key: String,
    pub chain: Vec<ChainedEvent>,
    pub chain_head: Vec<u8>,
    pub chain_depth: u64,
    pub sealed: bool,
}

impl EventChain {
    pub fn new(domain_id: uuid::Uuid, partition_key: impl Into<String>) -> Self {
        Self {
            domain_id,
            partition_key: partition_key.into(),
            chain: Vec::new(),
            chain_head: Sha256::new().chain_update(b"INWP_GENESIS").finalize().to_vec(),
            chain_depth: 0,
            sealed: false,
        }
    }

    pub fn append(&mut self, payload: Vec<u8>, node_id: uuid::Uuid) -> SyncResult<&ChainedEvent> {
        if self.sealed {
            return Err(crate::error::SyncEngineError::Internal(
                "Cannot append to sealed event chain".into(),
            ));
        }

        let event = ChainedEvent::new(
            self.chain_head.clone(),
            payload,
            node_id,
            self.domain_id,
            self.chain_depth,
        );

        self.chain_head = event.event_hash.clone();
        self.chain_depth += 1;
        self.chain.push(event);

        Ok(self.chain.last().unwrap())
    }

    pub fn verify_integrity(&self) -> ChainVerification {
        let mut issues = Vec::new();

        if self.chain.is_empty() {
            return ChainVerification {
                valid: true,
                verified_events: 0,
                issues: Vec::new(),
            };
        }

        let genesis = Sha256::new().chain_update(b"INWP_GENESIS").finalize().to_vec();
        if self.chain[0].previous_hash != genesis {
            issues.push("First event does not link to genesis".into());
        }

        for i in 1..self.chain.len() {
            if self.chain[i].previous_hash != self.chain[i - 1].event_hash {
                issues.push(format!(
                    "Chain break at event {} (depth {})",
                    self.chain[i].event_id, i
                ));
            }

            if !self.chain[i].verify_chain_integrity() {
                issues.push(format!(
                    "Event integrity violation at depth {}: {}",
                    i, self.chain[i].event_id
                ));
            }
        }

        ChainVerification {
            valid: issues.is_empty(),
            verified_events: self.chain.len() as u64,
            issues,
        }
    }

    pub fn replay_from(&self, from_depth: u64) -> Vec<&ChainedEvent> {
        self.chain.iter()
            .filter(|e| e.chain_depth >= from_depth)
            .collect()
    }

    pub fn seal(&mut self) {
        self.sealed = true;
    }
}

#[derive(Debug)]
pub struct ChainVerification {
    pub valid: bool,
    pub verified_events: u64,
    pub issues: Vec<String>,
}
