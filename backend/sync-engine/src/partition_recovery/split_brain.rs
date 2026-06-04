use super::{PartitionRecoveryContext, PartitionState};
use crate::core::merkle::MerkleTree;
use crate::error::{SyncEngineError, SyncResult};
use tracing::info;

pub struct SplitBrainResolver {
    max_healing_attempts: u32,
}

impl SplitBrainResolver {
    pub fn new(max_healing_attempts: u32) -> Self {
        Self {
            max_healing_attempts,
        }
    }

    pub fn detect_split_brain(
        &self,
        local_tree: &MerkleTree,
        remote_trees: &[&MerkleTree],
        partition_key: &str,
    ) -> SplitBrainDiagnosis {
        let mut divergent = Vec::new();

        for (i, remote) in remote_trees.iter().enumerate() {
            if local_tree.merkle_root() != remote.merkle_root() {
                let diff = local_tree.diff(remote);
                if !diff.is_empty() {
                    divergent.push(DivergentPeer {
                        peer_index: i,
                        merkle_root: remote.merkle_root().to_vec(),
                        divergent_records: diff.len(),
                        divergence_depth: compute_divergence_depth(local_tree, remote),
                    });
                }
            }
        }

        SplitBrainDiagnosis {
            partition_key: partition_key.to_string(),
            is_split_brain: divergent.len() > 1,
            local_root: local_tree.merkle_root().to_vec(),
            divergent_peers: divergent,
            total_peers: remote_trees.len(),
        }
    }

    pub fn resolve_split_brain(
        &self,
        context: &mut PartitionRecoveryContext,
        diagnosis: &SplitBrainDiagnosis,
    ) -> SyncResult<HealingPlan> {
        if context.recovery_attempts >= self.max_healing_attempts {
            return Err(SyncEngineError::Recovery(format!(
                "Split-brain recovery exceeded max attempts for partition {}",
                context.partition_key
            )));
        }

        context.state = PartitionState::Healing;
        context.recovery_attempts += 1;

        // Find the authoritative version (majority or highest authority)
        let authoritative = diagnosis
            .divergent_peers
            .iter()
            .max_by_key(|p| p.divergent_records);

        let authoritiative_root = authoritative
            .map(|a| a.merkle_root.clone())
            .unwrap_or_else(|| diagnosis.local_root.clone());

        info!(
            partition = %context.partition_key,
            divergent_peers = diagnosis.divergent_peers.len(),
            authoritative_root = %hex::encode(&authoritiative_root),
            "Split-brain healing plan generated"
        );

        Ok(HealingPlan {
            partition_key: context.partition_key.clone(),
            authoritative_merkle_root: authoritiative_root,
            peers_to_reconcile: diagnosis.divergent_peers.len() as u32,
            strategy: HealingStrategy::AuthoritativeOverride,
        })
    }
}

#[derive(Debug)]
pub struct SplitBrainDiagnosis {
    pub partition_key: String,
    pub is_split_brain: bool,
    pub local_root: Vec<u8>,
    pub divergent_peers: Vec<DivergentPeer>,
    pub total_peers: usize,
}

#[derive(Debug)]
pub struct DivergentPeer {
    pub peer_index: usize,
    pub merkle_root: Vec<u8>,
    pub divergent_records: usize,
    pub divergence_depth: u32,
}

#[derive(Debug)]
pub struct HealingPlan {
    pub partition_key: String,
    pub authoritative_merkle_root: Vec<u8>,
    pub peers_to_reconcile: u32,
    pub strategy: HealingStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealingStrategy {
    AuthoritativeOverride,
    MergeAll,
    LatestTimestamp,
    ManualIntervention,
}

fn compute_divergence_depth(local: &MerkleTree, remote: &MerkleTree) -> u32 {
    let local_keys: std::collections::HashSet<&str> =
        local.leaves.keys().map(|s| s.as_str()).collect();
    let remote_keys: std::collections::HashSet<&str> =
        remote.leaves.keys().map(|s| s.as_str()).collect();

    let symmetric_diff = local_keys.symmetric_difference(&remote_keys).count();
    symmetric_diff as u32
}
