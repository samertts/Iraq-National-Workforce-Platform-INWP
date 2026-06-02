use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    pub partition_key: String,
    pub root_hash: Vec<u8>,
    pub leaves: BTreeMap<String, Vec<u8>>,
    pub nodes: BTreeMap<String, Vec<u8>>,
    pub leaf_count: usize,
    pub height: u32,
}

impl MerkleTree {
    pub fn new(partition_key: impl Into<String>) -> Self {
        Self {
            partition_key: partition_key.into(),
            root_hash: Vec::new(),
            leaves: BTreeMap::new(),
            nodes: BTreeMap::new(),
            leaf_count: 0,
            height: 0,
        }
    }

    pub fn insert(&mut self, record_id: impl Into<String>, data: &[u8]) {
        let leaf_key = self.leaf_path(&record_id.into());
        let leaf_hash = hash_leaf(data);
        self.leaves.insert(leaf_key.clone(), leaf_hash.clone());
        self.leaf_count = self.leaves.len();
        self.rebuild();
    }

    pub fn remove(&mut self, record_id: &str) {
        let leaf_key = self.leaf_path(record_id);
        self.leaves.remove(&leaf_key);
        self.leaf_count = self.leaves.len();
        self.rebuild();
    }

    pub fn contains(&self, record_id: &str) -> bool {
        let leaf_key = self.leaf_path(record_id);
        self.leaves.contains_key(&leaf_key)
    }

    pub fn get_leaf_hash(&self, record_id: &str) -> Option<&Vec<u8>> {
        let leaf_key = self.leaf_path(record_id);
        self.leaves.get(&leaf_key)
    }

    pub fn rebuild(&mut self) {
        if self.leaves.is_empty() {
            self.root_hash = Vec::new();
            self.nodes.clear();
            self.height = 0;
            return;
        }

        self.nodes.clear();

        let mut level_nodes: Vec<(String, Vec<u8>)> = self.leaves.iter()
            .map(|(path, hash)| (path.clone(), hash.clone()))
            .collect();

        let mut height = 0_u32;
        while level_nodes.len() > 1 {
            let mut next_level = Vec::new();
            let prefix = if height == 0 {
                format!("{}/node", self.partition_key)
            } else {
                level_nodes[0].0.rsplit_once('/').map(|(p, _)| format!("{}/node", p)).unwrap_or_else(|| format!("{}/node", self.partition_key))
            };

            for chunk in level_nodes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    [chunk[0].1.as_slice(), chunk[1].1.as_slice()].concat()
                } else {
                    chunk[0].1.clone()
                };
                let parent_hash = hash_node(&combined);
                let parent_idx = next_level.len();
                let parent_key = format!("{}/{}", prefix, parent_idx);
                self.nodes.insert(parent_key.clone(), parent_hash.clone());
                next_level.push((parent_key, parent_hash));
            }

            level_nodes = next_level;
            height += 1;
        }

        self.root_hash = level_nodes.into_iter().next().map(|(_, h)| h).unwrap_or_default();
        self.height = height;
    }

    pub fn diff(&self, other: &MerkleTree) -> Vec<String> {
        if self.root_hash == other.root_hash && !self.root_hash.is_empty() {
            return Vec::new();
        }

        let mut divergent = Vec::new();
        let self_keys: HashSet<&String> = self.leaves.keys().collect();
        let other_keys: HashSet<&String> = other.leaves.keys().collect();

        for key in self_keys.symmetric_difference(&other_keys) {
            let record_id = self.record_id_from_path(key)
                .or_else(|| other.record_id_from_path(key));
            if let Some(id) = record_id {
                divergent.push(id);
            }
        }

        for key in self_keys.intersection(&other_keys) {
            let self_hash = &self.leaves[key.as_str()];
            let other_hash = &other.leaves[key.as_str()];
            if self_hash != other_hash {
                if let Some(record_id) = self.record_id_from_path(key) {
                    divergent.push(record_id);
                }
            }
        }

        divergent
    }

    pub fn compute_diff_since(&self, from_root: &[u8]) -> Vec<String> {
        if from_root.is_empty() {
            return self.leaves.keys()
                .filter_map(|k| self.record_id_from_path(k))
                .collect();
        }

        if self.root_hash == from_root {
            return Vec::new();
        }

        self.leaves.keys()
            .filter_map(|k| self.record_id_from_path(k))
            .collect()
    }

    pub fn generate_proof(&self, record_id: &str) -> Option<MerkleProof> {
        let leaf_key = self.leaf_path(record_id);
        let leaf_hash = self.leaves.get(&leaf_key)?.clone();

        // Build a sorted list of leaf keys and find the index of our target
        let sorted_leaves: Vec<&String> = {
            let mut keys: Vec<&String> = self.leaves.keys().collect();
            keys.sort();
            keys
        };
        let leaf_idx = sorted_leaves.iter().position(|k| *k == &leaf_key)? as u32;

        let mut siblings = Vec::new();

        // Reconstruct the level structure to find siblings
        // Collect hashes for each level like rebuild() does
        let mut level_hashes: Vec<Vec<Vec<u8>>> = Vec::new();
        let mut current_hashes: Vec<Vec<u8>> = sorted_leaves.iter()
            .map(|k| self.leaves[k.as_str()].clone())
            .collect();
        level_hashes.push(current_hashes.clone());

        while current_hashes.len() > 1 {
            let mut next = Vec::new();
            for chunk in current_hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    [chunk[0].as_slice(), chunk[1].as_slice()].concat()
                } else {
                    chunk[0].clone()
                };
                next.push(hash_node(&combined));
            }
            level_hashes.push(next.clone());
            current_hashes = next;
        }

        // Walk up the levels collecting siblings
        let mut current_idx = leaf_idx;
        for level_hashes in level_hashes.iter().take(level_hashes.len().saturating_sub(1)) {
            let sibling_idx = if current_idx.is_multiple_of(2) { current_idx + 1 } else { current_idx - 1 };
            if (sibling_idx as usize) < level_hashes.len() {
                siblings.push(level_hashes[sibling_idx as usize].clone());
            }
            current_idx /= 2;
        }

        Some(MerkleProof {
            partition_key: self.partition_key.clone(),
            record_id: record_id.to_string(),
            siblings,
            index: leaf_idx,
            leaf_hash,
        })
    }

    pub fn verify_proof(proof: &MerkleProof, root: &[u8]) -> bool {
        if root.is_empty() {
            return false;
        }

        let mut current = proof.leaf_hash.clone();
        for sibling in &proof.siblings {
            let combined = if proof.index.is_multiple_of(2) {
                [current.as_slice(), sibling.as_slice()].concat()
            } else {
                [sibling.as_slice(), current.as_slice()].concat()
            };
            current = hash_node(&combined);
        }

        current == root
    }

    pub fn merkle_root(&self) -> &[u8] {
        &self.root_hash
    }

    fn leaf_path(&self, record_id: &str) -> String {
        format!("{}/leaf/{}", self.partition_key, record_id)
    }

    pub(crate) fn record_id_from_path(&self, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 3 && parts[parts.len() - 2] == "leaf" {
            Some(parts[parts.len() - 1].to_string())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub partition_key: String,
    pub record_id: String,
    pub siblings: Vec<Vec<u8>>,
    pub index: u32,
    pub leaf_hash: Vec<u8>,
}

pub fn hash_leaf(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b"leaf:");
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn hash_node(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b"node:");
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn compute_record_hash(record_id: &str, record_type: &str, payload: &[u8], version: u64) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(record_id.as_bytes());
    hasher.update(b":");
    hasher.update(record_type.as_bytes());
    hasher.update(b":");
    hasher.update(payload);
    hasher.update(b":");
    hasher.update(version.to_le_bytes());
    hasher.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_tree() {
        let tree = MerkleTree::new("test-partition");
        assert!(tree.root_hash.is_empty());
        assert_eq!(tree.leaf_count, 0);
    }

    #[test]
    fn test_single_leaf() {
        let mut tree = MerkleTree::new("test");
        tree.insert("record-1", b"data-1");
        assert_eq!(tree.leaf_count, 1);
        assert!(!tree.root_hash.is_empty());
    }

    #[test]
    fn test_multiple_leaves() {
        let mut tree = MerkleTree::new("test");
        tree.insert("r1", b"data1");
        tree.insert("r2", b"data2");
        tree.insert("r3", b"data3");
        assert_eq!(tree.leaf_count, 3);
        assert!(!tree.root_hash.is_empty());
    }

    #[test]
    fn test_diff_no_changes() {
        let mut t1 = MerkleTree::new("p");
        t1.insert("r1", b"data1");
        t1.insert("r2", b"data2");

        let mut t2 = MerkleTree::new("p");
        t2.insert("r1", b"data1");
        t2.insert("r2", b"data2");

        assert!(t1.diff(&t2).is_empty());
    }

    #[test]
    fn test_diff_with_changes() {
        let mut t1 = MerkleTree::new("p");
        t1.insert("r1", b"data1");
        t1.insert("r2", b"data2");

        let mut t2 = MerkleTree::new("p");
        t2.insert("r1", b"modified");
        t2.insert("r2", b"data2");

        let diff = t1.diff(&t2);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0], "r1");
    }

    #[test]
    fn test_diff_extra_leaf() {
        let mut t1 = MerkleTree::new("p");
        t1.insert("r1", b"data1");

        let mut t2 = MerkleTree::new("p");
        t2.insert("r1", b"data1");
        t2.insert("r2", b"data2");

        let diff = t1.diff(&t2);
        assert_eq!(diff.len(), 1);
    }

    #[test]
    fn test_proof_verification() {
        let mut tree = MerkleTree::new("p");
        tree.insert("r1", b"data1");
        tree.insert("r2", b"data2");
        tree.insert("r3", b"data3");

        let root = tree.merkle_root().to_vec();
        let proof = tree.generate_proof("r1").unwrap();
        assert!(MerkleTree::verify_proof(&proof, &root));

        let wrong_root = hash_node(b"wrong");
        assert!(!MerkleTree::verify_proof(&proof, &wrong_root));
    }

    #[test]
    fn test_remove() {
        let mut tree = MerkleTree::new("p");
        tree.insert("r1", b"data1");
        tree.insert("r2", b"data2");
        let root_before = tree.merkle_root().to_vec();
        tree.remove("r1");
        assert_ne!(tree.merkle_root().to_vec(), root_before);
        assert_eq!(tree.leaf_count, 1);
    }

    #[test]
    fn test_compute_record_hash() {
        let h1 = compute_record_hash("id1", "ClockEvent", b"payload", 1);
        let h2 = compute_record_hash("id1", "ClockEvent", b"payload", 1);
        assert_eq!(h1, h2);

        let h3 = compute_record_hash("id1", "ClockEvent", b"payload", 2);
        assert_ne!(h1, h3);
    }
}
