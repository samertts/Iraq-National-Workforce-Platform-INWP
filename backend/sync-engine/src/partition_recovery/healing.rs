use super::{PartitionRecoveryContext, PartitionState};
use crate::core::merkle::MerkleTree;
use crate::error::{SyncEngineError, SyncResult};
use tracing::info;

pub struct PartitionHealer {
    max_heal_attempts: u32,
}

impl PartitionHealer {
    pub fn new(max_heal_attempts: u32) -> Self {
        Self { max_heal_attempts }
    }

    pub fn heal_partition(
        &self,
        context: &mut PartitionRecoveryContext,
        local_tree: &MerkleTree,
        reference_tree: &MerkleTree,
    ) -> SyncResult<HealOutcome> {
        if context.recovery_attempts >= self.max_heal_attempts {
            return Err(SyncEngineError::Recovery(format!(
                "Healing exceeded max attempts for partition {}",
                context.partition_key
            )));
        }

        let diff = local_tree.diff(reference_tree);

        if diff.is_empty() {
            context.state = PartitionState::Recovered;
            return Ok(HealOutcome {
                partition_key: context.partition_key.clone(),
                records_to_reconcile: 0,
                healed: true,
            });
        }

        context.state = PartitionState::Healing;
        context.recovery_attempts += 1;

        info!(
            partition = %context.partition_key,
            divergent_records = diff.len(),
            attempt = context.recovery_attempts,
            "Healing partition divergence"
        );

        Ok(HealOutcome {
            partition_key: context.partition_key.clone(),
            records_to_reconcile: diff.len() as u64,
            healed: false,
        })
    }

    pub fn isolate_corruption(
        &self,
        context: &mut PartitionRecoveryContext,
        corrupted_records: &[String],
    ) -> SyncResult<()> {
        context.state = PartitionState::Isolated;
        info!(
            partition = %context.partition_key,
            corrupted_count = corrupted_records.len(),
            "Partition isolated due to corruption"
        );
        Ok(())
    }
}

#[derive(Debug)]
pub struct HealOutcome {
    pub partition_key: String,
    pub records_to_reconcile: u64,
    pub healed: bool,
}
