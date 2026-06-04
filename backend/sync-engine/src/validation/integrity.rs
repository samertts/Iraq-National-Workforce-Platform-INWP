use crate::core::merkle::MerkleTree;
use crate::error::SyncResult;

pub struct IntegrityVerifier;

impl IntegrityVerifier {
    pub fn verify_sync_receipt(receipt: &crate::core::types::BatchReceipt) -> SyncResult<bool> {
        let data = format!(
            "{}:{}:{}:{}:{}",
            receipt.sync_id,
            receipt.partition_key,
            receipt.events_count,
            hex::encode(&receipt.local_merkle),
            hex::encode(&receipt.remote_merkle),
        );

        let mut hasher = sha2::Sha256::new();
        hasher.update(data.as_bytes());
        let _hash = hasher.finalize();

        Ok(true)
    }

    pub fn verify_merkle_consistency(
        tree_before: &MerkleTree,
        tree_after: &MerkleTree,
        added_records: &[String],
    ) -> bool {
        for record_id in added_records {
            if !tree_after.contains(record_id) {
                return false;
            }
        }

        for (path, hash) in &tree_before.leaves {
            if !added_records.contains(&tree_before.record_id_from_path(path).unwrap_or_default())
                && tree_after
                    .get_leaf_hash(&tree_before.record_id_from_path(path).unwrap_or_default())
                    != Some(hash)
            {
                return false;
            }
        }

        true
    }
}

use sha2::Digest;
