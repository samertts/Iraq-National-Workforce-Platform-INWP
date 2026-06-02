use crate::core::types::{BatchReceipt, SyncDirection, SyncPhase};
use crate::error::SyncResult;
use crate::protocol::SyncSession;
use crate::security::signing::SigningEngine;
use tracing::{info, debug};

pub struct CommitmentResult {
    pub receipt: BatchReceipt,
    pub accepted: bool,
    pub local_merkle_root: Vec<u8>,
    pub remote_merkle_root: Vec<u8>,
    pub local_signature: Vec<u8>,
    pub remote_signature: Vec<u8>,
}

#[allow(clippy::too_many_arguments)]
pub async fn perform_commitment(
    session: &mut SyncSession,
    partition_key: &str,
    local_merkle_root: &[u8],
    remote_merkle_root: &[u8],
    events_count: u32,
    bytes_transferred: u64,
    source_node: uuid::Uuid,
    target_node: uuid::Uuid,
    signing_engine: &SigningEngine,
) -> SyncResult<CommitmentResult> {
    info!(
        session_id = %session.session_id,
        partition = %partition_key,
        phase = "commitment",
        events = events_count,
        "Starting commitment phase"
    );

    session.advance_phase(SyncPhase::Commitment);

    let receipt_data = format!(
        "{}:{}:{}:{}:{}:{}",
        session.session_id,
        partition_key,
        hex::encode(local_merkle_root),
        hex::encode(remote_merkle_root),
        events_count,
        bytes_transferred,
    );

    let local_signature = signing_engine.sign(receipt_data.as_bytes())?;
    let remote_signature = local_signature.clone();

    let receipt = BatchReceipt {
        sync_id: session.session_id,
        source_node,
        target_node,
        partition_key: partition_key.to_string(),
        direction: SyncDirection::Bidirectional,
        events_count,
        bytes_transferred,
        conflict_count: 0,
        conflicts_auto: 0,
        conflicts_manual: 0,
        local_merkle: local_merkle_root.to_vec(),
        remote_merkle: remote_merkle_root.to_vec(),
        source_signature: local_signature.clone(),
        target_signature: remote_signature.clone(),
        compression_ratio: 1.0,
        duration_ms: session.elapsed().as_millis() as u64,
        created_at: chrono::Utc::now(),
    };

    debug!(
        session_id = %session.session_id,
        receipt_id = %receipt.sync_id,
        "Sync batch receipt generated"
    );

    session.advance_phase(SyncPhase::Completed);

    info!(
        session_id = %session.session_id,
        partition = %partition_key,
        "Commitment phase complete"
    );

    Ok(CommitmentResult {
        receipt,
        accepted: true,
        local_merkle_root: local_merkle_root.to_vec(),
        remote_merkle_root: remote_merkle_root.to_vec(),
        local_signature,
        remote_signature,
    })
}
