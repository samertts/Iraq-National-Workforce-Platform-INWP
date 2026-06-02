pub mod safety;
pub mod validator;

pub use safety::*;
pub use validator::*;

use serde::{Deserialize, Serialize};

/// Complete verification context for a replay operation
#[derive(Debug, Clone)]
pub struct ReplayVerificationContext {
    pub partition_key: String,
    pub from_checkpoint: Vec<u8>,
    pub to_checkpoint: Vec<u8>,
    pub expected_event_count: u64,
    pub original_merkle_root: Vec<u8>,
    pub verify_signatures: bool,
    pub verify_chain: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayVerificationResult {
    pub valid: bool,
    pub events_verified: u64,
    pub events_failed: u64,
    pub merkle_match: bool,
    pub signature_errors: u32,
    pub chain_errors: u32,
    pub checkpoint_match: bool,
    pub issues: Vec<String>,
}
