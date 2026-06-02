use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub node_id: uuid::Uuid,
    pub partition_key: String,
    pub merkle_root: Vec<u8>,
    pub last_sync_at: chrono::DateTime<chrono::Utc>,
    pub synced_events: u64,
    pub last_error: Option<String>,
}

impl Checkpoint {
    pub fn new(node_id: uuid::Uuid, partition_key: impl Into<String>) -> Self {
        Self {
            node_id,
            partition_key: partition_key.into(),
            merkle_root: Vec::new(),
            last_sync_at: chrono::Utc::now(),
            synced_events: 0,
            last_error: None,
        }
    }

    pub fn advance(&mut self, merkle_root: Vec<u8>, events_count: u64) {
        self.merkle_root = merkle_root;
        self.last_sync_at = chrono::Utc::now();
        self.synced_events += events_count;
        self.last_error = None;
    }

    pub fn is_initial(&self) -> bool {
        self.merkle_root.is_empty() || self.synced_events == 0
    }
}
