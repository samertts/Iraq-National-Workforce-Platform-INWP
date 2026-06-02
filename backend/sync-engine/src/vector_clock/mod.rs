pub mod clock;

pub use clock::*;

use crate::core::version::VersionVector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dotted Version Vector — tracks per-node versions with causal history dots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DottedVersionVector {
    pub base: VersionVector,
    pub dots: HashMap<uuid::Uuid, Vec<u64>>,
    pub causal_history: Vec<CausalEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEvent {
    pub event_id: uuid::Uuid,
    pub node_id: uuid::Uuid,
    pub counter: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub depends_on: Vec<uuid::Uuid>,
}

impl DottedVersionVector {
    pub fn new(node_id: uuid::Uuid) -> Self {
        Self {
            base: VersionVector::new(node_id),
            dots: HashMap::new(),
            causal_history: Vec::new(),
        }
    }

    pub fn record_event(&mut self, node_id: uuid::Uuid, event_id: uuid::Uuid) {
        let counter = self.base.versions.entry(node_id).or_insert(0);
        *counter += 1;

        let dots = self.dots.entry(node_id).or_default();
        dots.push(*counter);

        let deps: Vec<uuid::Uuid> = self.causal_history.last()
            .map(|last| vec![last.event_id])
            .unwrap_or_default();

        self.causal_history.push(CausalEvent {
            event_id,
            node_id,
            counter: *counter,
            timestamp: chrono::Utc::now(),
            depends_on: deps,
        });
    }

    pub fn happens_before(&self, other: &DottedVersionVector) -> bool {
        // All our versions <= theirs, and at least one < theirs
        let mut less = false;
        let mut greater = false;

        for (node, &count) in &self.base.versions {
            let other_count = other.base.versions.get(node).copied().unwrap_or(0);
            if count > other_count {
                greater = true;
            }
            if count < other_count {
                less = true;
            }
        }

        for (node, &count) in &other.base.versions {
            let self_count = self.base.versions.get(node).copied().unwrap_or(0);
            if count > self_count {
                less = true;
            }
            if count < self_count {
                greater = true;
            }
        }

        less && !greater
    }

    pub fn concurrent(&self, other: &DottedVersionVector) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }

    pub fn merge(&mut self, other: &DottedVersionVector) {
        self.base.merge(&other.base);

        for (node, dots) in &other.dots {
            let entry = self.dots.entry(*node).or_default();
            for dot in dots {
                if !entry.contains(dot) {
                    entry.push(*dot);
                }
            }
            entry.sort();
        }

        // Merge causal history
        for event in &other.causal_history {
            if !self.causal_history.iter().any(|e| e.event_id == event.event_id) {
                self.causal_history.push(event.clone());
            }
        }
    }

    pub fn causal_length(&self) -> usize {
        self.causal_history.len()
    }
}
