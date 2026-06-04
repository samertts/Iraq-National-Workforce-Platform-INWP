use crate::core::merkle::{compute_record_hash, MerkleTree};
use crate::error::SyncResult;
use sha2::{Digest, Sha256};
use tracing::warn;

pub struct CorruptionDetector {
    check_interval: chrono::Duration,
}

impl CorruptionDetector {
    pub fn new(check_interval_secs: u64) -> Self {
        Self {
            check_interval: chrono::Duration::seconds(check_interval_secs as i64),
        }
    }

    pub fn detect_merkle_corruption(
        &self,
        expected_root: &[u8],
        current_tree: &MerkleTree,
    ) -> SyncResult<CorruptionReport> {
        let mut issues = Vec::new();

        if expected_root.is_empty() {
            return Ok(CorruptionReport {
                is_corrupted: false,
                issues: Vec::new(),
                verified_records: 0,
            });
        }

        let computed_root = current_tree.merkle_root().to_vec();

        if computed_root != expected_root {
            issues.push(CorruptionIssue {
                severity: CorruptionSeverity::High,
                issue_type: "merkle_root_mismatch".into(),
                description: format!(
                    "Merkle root mismatch: expected {}, computed {}",
                    hex::encode(expected_root),
                    hex::encode(&computed_root)
                ),
                affected_partition: current_tree.partition_key.clone(),
                affected_records: current_tree.leaf_count,
            });
        }

        let mut verified = 0_u64;
        for (path, stored_hash) in &current_tree.leaves {
            let parts: Vec<&str> = path.split('/').collect();
            if let Some(record_id) = parts.last() {
                if let Some(record_hash) = current_tree.get_leaf_hash(record_id) {
                    if record_hash != stored_hash {
                        issues.push(CorruptionIssue {
                            severity: CorruptionSeverity::Critical,
                            issue_type: "leaf_hash_mismatch".into(),
                            description: format!("Leaf hash mismatch for record {}", record_id),
                            affected_partition: current_tree.partition_key.clone(),
                            affected_records: 1,
                        });
                    }
                    verified += 1;
                }
            }
        }

        let is_corrupted = !issues.is_empty();

        if is_corrupted {
            warn!(
                issue_count = issues.len(),
                verified_records = verified,
                "Corruption detected in Merkle tree"
            );
        }

        Ok(CorruptionReport {
            is_corrupted,
            issues,
            verified_records: verified,
        })
    }

    pub fn verify_record_chain(
        &self,
        records: &[crate::core::types::SyncRecord],
    ) -> Vec<CorruptionIssue> {
        let mut issues = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for record in records {
            if !seen_ids.insert(record.record_id.clone()) {
                issues.push(CorruptionIssue {
                    severity: CorruptionSeverity::Medium,
                    issue_type: "duplicate_record".into(),
                    description: format!("Duplicate record ID: {}", record.record_id),
                    affected_partition: String::new(),
                    affected_records: 1,
                });
            }

            let computed_hash = compute_record_hash(
                &record.record_id,
                &record.record_type,
                &record.payload,
                record.version_vector.local_timestamp,
            );

            if !record.signature.is_empty() {
                let mut hasher = Sha256::new();
                hasher.update(&computed_hash);
                let verification_hash = hasher.finalize().to_vec();
                if verification_hash != computed_hash {
                    issues.push(CorruptionIssue {
                        severity: CorruptionSeverity::High,
                        issue_type: "signature_mismatch".into(),
                        description: format!("Record hash mismatch for {}", record.record_id),
                        affected_partition: String::new(),
                        affected_records: 1,
                    });
                }
            }
        }

        issues
    }
}

#[derive(Debug)]
pub struct CorruptionReport {
    pub is_corrupted: bool,
    pub issues: Vec<CorruptionIssue>,
    pub verified_records: u64,
}

#[derive(Debug)]
pub struct CorruptionIssue {
    pub severity: CorruptionSeverity,
    pub issue_type: String,
    pub description: String,
    pub affected_partition: String,
    pub affected_records: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorruptionSeverity {
    Low,
    Medium,
    High,
    Critical,
}
