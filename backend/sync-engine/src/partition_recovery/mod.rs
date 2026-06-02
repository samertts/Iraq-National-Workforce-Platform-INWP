pub mod healing;
pub mod split_brain;

pub use healing::*;
pub use split_brain::*;

use serde::{Deserialize, Serialize};

/// State of a partition during recovery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartitionState {
    Healthy,
    Diverged,
    SplitBrain,
    Healing,
    Recovered,
    Isolated,
    Corrupted,
}

impl PartitionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Diverged => "diverged",
            Self::SplitBrain => "split_brain",
            Self::Healing => "healing",
            Self::Recovered => "recovered",
            Self::Isolated => "isolated",
            Self::Corrupted => "corrupted",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionRecoveryContext {
    pub partition_key: String,
    pub state: PartitionState,
    pub diverged_since: Option<chrono::DateTime<chrono::Utc>>,
    pub local_checkpoint: Vec<u8>,
    pub remote_checkpoint: Vec<u8>,
    pub divergence_depth: u64,
    pub node_count: u32,
    pub recovery_attempts: u32,
}
