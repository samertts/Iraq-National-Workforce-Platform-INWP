use crate::core::merkle::MerkleTree;
use crate::error::SyncResult;

pub struct CheckpointRecoverer {
    checkpoint_interval_events: u64,
    checkpoint_interval_secs: u64,
}

impl CheckpointRecoverer {
    pub fn new(checkpoint_interval_events: u64, checkpoint_interval_secs: u64) -> Self {
        Self {
            checkpoint_interval_events,
            checkpoint_interval_secs,
        }
    }

    pub fn should_checkpoint(
        &self,
        events_since_last: u64,
        time_since_last: chrono::Duration,
    ) -> bool {
        events_since_last >= self.checkpoint_interval_events
            || time_since_last.num_seconds() >= self.checkpoint_interval_secs as i64
    }

    pub fn recover_from_checkpoint(
        &self,
        checkpoint: &crate::core::Checkpoint,
        current_tree: &MerkleTree,
    ) -> SyncResult<Vec<String>> {
        if checkpoint.is_initial() {
            return Ok(current_tree
                .leaves
                .keys()
                .filter_map(|k| {
                    let parts: Vec<&str> = k.split('/').collect();
                    parts.last().map(|s| s.to_string())
                })
                .collect());
        }

        let divergent = current_tree.compute_diff_since(&checkpoint.merkle_root);

        Ok(divergent)
    }

    pub fn verify_checkpoint_integrity(
        &self,
        stored_merkle_root: &[u8],
        computed_tree: &MerkleTree,
    ) -> SyncResult<bool> {
        if stored_merkle_root.is_empty() && computed_tree.merkle_root().is_empty() {
            return Ok(true);
        }
        if stored_merkle_root.is_empty() || computed_tree.merkle_root().is_empty() {
            return Ok(false);
        }
        Ok(stored_merkle_root == computed_tree.merkle_root())
    }
}
