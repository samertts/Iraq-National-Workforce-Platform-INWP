use crate::core::types::{SyncPhase, SyncRecord};
use crate::error::SyncResult;
use crate::protocol::codec::SyncCodec;
use crate::protocol::SyncSession;
use tracing::{info, debug};

pub struct DeltaTransferResult {
    pub records_received: Vec<SyncRecord>,
    pub records_sent: u32,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub has_more: bool,
    pub next_batch_seq: u32,
}

pub async fn transfer_delta(
    session: &mut SyncSession,
    partition_key: &str,
    divergent_records: &[String],
    _codec: &SyncCodec,
) -> SyncResult<DeltaTransferResult> {
    info!(
        session_id = %session.session_id,
        partition = %partition_key,
        divergent_count = divergent_records.len(),
        phase = "delta_transfer",
        "Starting delta transfer"
    );

    session.advance_phase(SyncPhase::DeltaTransfer);

    let batch_size = 1000.min(divergent_records.len());
    let batch: Vec<String> = divergent_records.iter()
        .take(batch_size)
        .cloned()
        .collect();

    debug!(
        partition = %partition_key,
        batch_size = batch.len(),
        "Compressing delta batch"
    );

    let compressed_size = batch.len() as u64;
    session.add_transfer(compressed_size, batch.len() as u32);

    let has_more = batch_size < divergent_records.len();

    info!(
        partition = %partition_key,
        transferred = batch.len(),
        has_more = has_more,
        "Delta transfer phase complete"
    );

    Ok(DeltaTransferResult {
        records_received: Vec::new(),
        records_sent: batch.len() as u32,
        bytes_received: 0,
        bytes_sent: compressed_size,
        has_more,
        next_batch_seq: if has_more { 1 } else { 0 },
    })
}
