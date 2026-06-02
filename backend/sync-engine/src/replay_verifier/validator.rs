use super::{ReplayVerificationContext, ReplayVerificationResult};
use crate::core::merkle::{MerkleTree, compute_record_hash};
use crate::core::types::SyncRecord;
use crate::error::SyncResult;

pub struct ReplayValidator {
    max_batch_size: u32,
}

impl ReplayValidator {
    pub fn new(max_batch_size: u32) -> Self {
        Self { max_batch_size }
    }

    pub fn validate_replay_batch(
        &self,
        batch_seq: u32,
        records: &[SyncRecord],
        expected_count: u64,
    ) -> SyncResult<BatchValidation> {
        let mut issues = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        if records.len() as u64 > expected_count {
            issues.push(format!(
                "Batch {} exceeds expected count: {} > {}",
                batch_seq,
                records.len(),
                expected_count
            ));
        }

        for record in records {
            if !seen_ids.insert(record.record_id.clone()) {
                issues.push(format!(
                    "Duplicate record_id in replay batch {}: {}",
                    batch_seq, record.record_id
                ));
            }

            // Verify record hash
            let computed = compute_record_hash(
                &record.record_id,
                &record.record_type,
                &record.payload,
                record.version_vector.local_timestamp,
            );
            if !record.signature.is_empty() && record.signature != computed {
                issues.push(format!(
                    "Record hash mismatch for {} in batch {}",
                    record.record_id, batch_seq
                ));
            }
        }

        Ok(BatchValidation {
            valid: issues.is_empty(),
            issues,
            record_count: records.len() as u32,
        })
    }

    pub fn validate_complete_replay(
        &self,
        context: &ReplayVerificationContext,
        reconstructed_tree: &MerkleTree,
    ) -> ReplayVerificationResult {
        let mut issues = Vec::new();

        // Checkpoint match
        let checkpoint_match = if context.to_checkpoint.is_empty() {
            true
        } else {
            reconstructed_tree.merkle_root() == context.to_checkpoint
        };

        if !checkpoint_match {
            issues.push(format!(
                "Final merkle root {:?} does not match expected checkpoint {:?}",
                hex::encode(reconstructed_tree.merkle_root()),
                hex::encode(&context.to_checkpoint)
            ));
        }

        // Merkle match
        let merkle_match = if context.original_merkle_root.is_empty() {
            true
        } else {
            reconstructed_tree.merkle_root() == context.original_merkle_root
        };

        if !merkle_match {
            issues.push("Reconstructed merkle root differs from original".into());
        }

        ReplayVerificationResult {
            valid: checkpoint_match && merkle_match,
            events_verified: reconstructed_tree.leaf_count as u64,
            events_failed: 0,
            merkle_match,
            signature_errors: 0,
            chain_errors: 0,
            checkpoint_match,
            issues,
        }
    }
}

#[derive(Debug)]
pub struct BatchValidation {
    pub valid: bool,
    pub issues: Vec<String>,
    pub record_count: u32,
}
