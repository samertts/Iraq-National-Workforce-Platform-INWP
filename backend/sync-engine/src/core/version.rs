use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionVector {
    pub versions: HashMap<uuid::Uuid, u64>,
    pub local_timestamp: u64,
}

impl VersionVector {
    pub fn new(node_id: uuid::Uuid) -> Self {
        let mut versions = HashMap::new();
        versions.insert(node_id, 1);
        Self {
            versions,
            local_timestamp: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64,
        }
    }

    pub fn increment(&mut self, node_id: uuid::Uuid) {
        let counter = self.versions.entry(node_id).or_insert(0);
        *counter += 1;
        self.local_timestamp = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) as u64;
    }

    pub fn merge(&mut self, other: &VersionVector) {
        for (node, &count) in &other.versions {
            let entry = self.versions.entry(*node).or_insert(0);
            *entry = (*entry).max(count);
        }
        self.local_timestamp = self.local_timestamp.max(other.local_timestamp);
    }

    pub fn compare(&self, other: &VersionVector) -> VersionOrder {
        let mut self_greater = false;
        let mut other_greater = false;

        let all_nodes: std::collections::HashSet<&uuid::Uuid> =
            self.versions.keys().chain(other.versions.keys()).collect();

        for &node in &all_nodes {
            let self_v = self.versions.get(node).copied().unwrap_or(0);
            let other_v = other.versions.get(node).copied().unwrap_or(0);
            if self_v > other_v {
                self_greater = true;
            }
            if other_v > self_v {
                other_greater = true;
            }
        }

        match (self_greater, other_greater) {
            (true, false) => VersionOrder::After,
            (false, true) => VersionOrder::Before,
            (true, true) => VersionOrder::Concurrent,
            (false, false) => VersionOrder::Equal,
        }
    }

    pub fn is_ancestor_of(&self, other: &VersionVector) -> bool {
        self.compare(other) == VersionOrder::Before
    }

    pub fn is_descendant_of(&self, other: &VersionVector) -> bool {
        self.compare(other) == VersionOrder::After
    }

    pub fn conflicts_with(&self, other: &VersionVector) -> bool {
        self.compare(other) == VersionOrder::Concurrent
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionOrder {
    Before,
    After,
    Equal,
    Concurrent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorClock {
    pub node_id: uuid::Uuid,
    pub counter: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl VectorClock {
    pub fn new(node_id: uuid::Uuid) -> Self {
        Self {
            node_id,
            counter: 0,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn tick(&mut self) {
        self.counter += 1;
        self.timestamp = chrono::Utc::now();
    }

    pub fn happend_before(&self, other: &VectorClock) -> bool {
        self.counter < other.counter && self.timestamp < other.timestamp
    }
}
