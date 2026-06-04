use crate::core::merkle::MerkleTree;
use crate::error::{SyncEngineError, SyncResult};
use crate::recovery::checkpoint::CheckpointRecoverer;
use tracing::{info, warn};

pub struct ReplayEngine {
    batch_size: u32,
    verify_merkle: bool,
    checkpoint_recoverer: CheckpointRecoverer,
}

impl ReplayEngine {
    pub fn new(
        batch_size: u32,
        verify_merkle: bool,
        checkpoint_recoverer: CheckpointRecoverer,
    ) -> Self {
        Self {
            batch_size,
            verify_merkle,
            checkpoint_recoverer,
        }
    }

    pub async fn start_replay(
        &self,
        from_checkpoint: &crate::core::Checkpoint,
        events: &[crate::core::types::SyncRecord],
    ) -> SyncResult<ReplayResult> {
        info!(
            partition = %from_checkpoint.partition_key,
            from_events = from_checkpoint.synced_events,
            total_events = events.len(),
            "Starting event replay"
        );

        let mut applied = 0_u32;
        let mut skipped = 0_u32;
        let mut batch_seq = 0_u32;
        let mut current_tree = MerkleTree::new(&from_checkpoint.partition_key);

        for chunk in events.chunks(self.batch_size as usize) {
            for event in chunk {
                let record_hash = crate::core::merkle::compute_record_hash(
                    &event.record_id,
                    &event.record_type,
                    &event.payload,
                    event.version_vector.local_timestamp,
                );

                if current_tree.get_leaf_hash(&event.record_id) == Some(&record_hash) {
                    skipped += 1;
                    continue;
                }

                current_tree.insert(&event.record_id, &event.payload);
                applied += 1;
            }

            if self.verify_merkle {
                let stored_root = from_checkpoint.merkle_root.as_slice();
                if !stored_root.is_empty() {
                    let integrity_ok = self
                        .checkpoint_recoverer
                        .verify_checkpoint_integrity(stored_root, &current_tree)?;
                    if !integrity_ok {
                        warn!(batch = batch_seq, "Merkle root mismatch during replay");
                        return Err(SyncEngineError::Corruption(format!(
                            "Merkle root mismatch at batch {} during replay",
                            batch_seq
                        )));
                    }
                }
            }

            batch_seq += 1;
        }

        info!(
            applied = applied,
            skipped = skipped,
            batches = batch_seq,
            "Replay completed"
        );

        Ok(ReplayResult {
            events_applied: applied,
            events_skipped: skipped,
            total_batches: batch_seq,
            final_merkle_root: current_tree.merkle_root().to_vec(),
            success: true,
        })
    }

    pub fn verify_replay_integrity(
        &self,
        original_events: &[crate::core::types::SyncRecord],
        replayed_tree: &MerkleTree,
    ) -> bool {
        for event in original_events {
            let hash = crate::core::merkle::compute_record_hash(
                &event.record_id,
                &event.record_type,
                &event.payload,
                event.version_vector.local_timestamp,
            );
            if !replayed_tree.contains(&event.record_id) {
                return false;
            }
            if replayed_tree.get_leaf_hash(&event.record_id) != Some(&hash) {
                return false;
            }
        }
        true
    }
}

#[derive(Debug)]
pub struct ReplayResult {
    pub events_applied: u32,
    pub events_skipped: u32,
    pub total_batches: u32,
    pub final_merkle_root: Vec<u8>,
    pub success: bool,
}
