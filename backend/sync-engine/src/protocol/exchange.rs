use crate::core::merkle::MerkleTree;
use crate::core::types::SyncPhase;
use crate::error::SyncResult;
use crate::protocol::SyncSession;
use tracing::{debug, info};

#[derive(Debug)]
pub struct MerkleExchangeResult {
    pub divergent_records: Vec<String>,
    pub local_root: Vec<u8>,
    pub remote_root: Vec<u8>,
    pub partitions_with_diffs: Vec<String>,
    pub complete: bool,
}

pub async fn perform_merkle_exchange(
    session: &mut SyncSession,
    partition_key: &str,
    local_tree: &MerkleTree,
    remote_tree: &MerkleTree,
) -> SyncResult<MerkleExchangeResult> {
    info!(
        session_id = %session.session_id,
        partition = %partition_key,
        phase = "merkle_exchange",
        "Starting Merkle exchange"
    );

    session.advance_phase(SyncPhase::MerkleExchange);

    if local_tree.leaf_count == 0 && remote_tree.leaf_count == 0 {
        return Ok(MerkleExchangeResult {
            divergent_records: Vec::new(),
            local_root: local_tree.merkle_root().to_vec(),
            remote_root: remote_tree.merkle_root().to_vec(),
            partitions_with_diffs: Vec::new(),
            complete: true,
        });
    }

    if local_tree.merkle_root() == remote_tree.merkle_root() {
        debug!(
            partition = %partition_key,
            "Merkle roots match, no changes needed"
        );
        return Ok(MerkleExchangeResult {
            divergent_records: Vec::new(),
            local_root: local_tree.merkle_root().to_vec(),
            remote_root: remote_tree.merkle_root().to_vec(),
            partitions_with_diffs: Vec::new(),
            complete: true,
        });
    }

    let divergent = local_tree.diff(remote_tree);

    info!(
        partition = %partition_key,
        divergent_count = divergent.len(),
        local_leaves = local_tree.leaf_count,
        remote_leaves = remote_tree.leaf_count,
        "Merkle exchange completed with divergences"
    );

    Ok(MerkleExchangeResult {
        divergent_records: divergent,
        local_root: local_tree.merkle_root().to_vec(),
        remote_root: remote_tree.merkle_root().to_vec(),
        partitions_with_diffs: if local_tree.merkle_root() != remote_tree.merkle_root() {
            vec![partition_key.to_string()]
        } else {
            Vec::new()
        },
        complete: true,
    })
}

pub fn should_recursive_descend(
    local_subtree: &[u8],
    remote_subtree: &[u8],
    depth: u32,
    max_depth: u32,
) -> bool {
    if local_subtree == remote_subtree {
        return false;
    }
    depth < max_depth
}
