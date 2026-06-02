use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrdtValue {
    Lww(LwwRegister),
    PnCounter(PnCounter),
    GSet(GSet),
    MvRegister(MvRegister),
    ORMap(ORMap),
}

impl CrdtValue {
    pub fn merge(&self, other: &CrdtValue) -> SyncResult<Self> {
        match (self, other) {
            (Self::Lww(a), Self::Lww(b)) => Ok(Self::Lww(a.merge(b))),
            (Self::PnCounter(a), Self::PnCounter(b)) => Ok(Self::PnCounter(a.merge(b))),
            (Self::GSet(a), Self::GSet(b)) => Ok(Self::GSet(a.merge(b))),
            (Self::MvRegister(a), Self::MvRegister(b)) => Ok(Self::MvRegister(a.merge(b))),
            (Self::ORMap(a), Self::ORMap(b)) => Ok(Self::ORMap(a.merge(b))),
            _ => Err(SyncEngineError::Reconciliation(
                "Cannot merge different CRDT types".into(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LwwRegister {
    pub value: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub node_id: uuid::Uuid,
}

impl LwwRegister {
    pub fn new(value: Vec<u8>, node_id: uuid::Uuid) -> Self {
        Self {
            value,
            timestamp: chrono::Utc::now(),
            node_id,
        }
    }

    pub fn merge(&self, other: &LwwRegister) -> Self {
        if self.timestamp >= other.timestamp {
            self.clone()
        } else {
            other.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnCounter {
    pub positive: HashMap<uuid::Uuid, i64>,
    pub negative: HashMap<uuid::Uuid, i64>,
}

impl Default for PnCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl PnCounter {
    pub fn new() -> Self {
        Self {
            positive: HashMap::new(),
            negative: HashMap::new(),
        }
    }

    pub fn value(&self) -> i64 {
        let p: i64 = self.positive.values().sum();
        let n: i64 = self.negative.values().sum();
        p - n
    }

    pub fn increment(&mut self, node_id: uuid::Uuid, amount: u64) {
        *self.positive.entry(node_id).or_insert(0) += amount as i64;
    }

    pub fn decrement(&mut self, node_id: uuid::Uuid, amount: u64) {
        *self.negative.entry(node_id).or_insert(0) += amount as i64;
    }

    pub fn merge(&self, other: &PnCounter) -> PnCounter {
        let mut merged = PnCounter::new();
        for (node, &count) in &self.positive {
            *merged.positive.entry(*node).or_insert(0) = count;
        }
        for (node, &count) in &other.positive {
            let entry = merged.positive.entry(*node).or_insert(0);
            *entry = (*entry).max(count);
        }
        for (node, &count) in &self.negative {
            *merged.negative.entry(*node).or_insert(0) = count;
        }
        for (node, &count) in &other.negative {
            let entry = merged.negative.entry(*node).or_insert(0);
            *entry = (*entry).max(count);
        }
        merged
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GSet {
    pub elements: HashSet<Vec<u8>>,
}

impl Default for GSet {
    fn default() -> Self {
        Self::new()
    }
}

impl GSet {
    pub fn new() -> Self {
        Self {
            elements: HashSet::new(),
        }
    }

    pub fn add(&mut self, element: Vec<u8>) {
        self.elements.insert(element);
    }

    pub fn contains(&self, element: &[u8]) -> bool {
        self.elements.contains(element)
    }

    pub fn merge(&self, other: &GSet) -> GSet {
        let mut merged = self.elements.clone();
        merged.extend(other.elements.clone());
        GSet { elements: merged }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MvRegister {
    pub values: HashMap<Vec<u8>, LwwRegister>,
}

impl Default for MvRegister {
    fn default() -> Self {
        Self::new()
    }
}

impl MvRegister {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn assign(&mut self, key: Vec<u8>, value: Vec<u8>, node_id: uuid::Uuid) {
        let reg = LwwRegister::new(value, node_id);
        self.values.insert(key, reg);
    }

    pub fn merge(&self, other: &MvRegister) -> MvRegister {
        let mut merged = self.values.clone();
        for (key, reg) in &other.values {
            match merged.get(key) {
                Some(existing) => {
                    merged.insert(key.clone(), existing.merge(reg));
                }
                None => {
                    merged.insert(key.clone(), reg.clone());
                }
            }
        }
        MvRegister { values: merged }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ORMap {
    pub entries: HashMap<String, CrdtValue>,
}

impl Default for ORMap {
    fn default() -> Self {
        Self::new()
    }
}

impl ORMap {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn merge(&self, other: &ORMap) -> ORMap {
        let mut merged = self.entries.clone();
        for (key, value) in &other.entries {
            match merged.get(key) {
                Some(existing) => {
                    if let Ok(merged_val) = existing.merge(value) {
                        merged.insert(key.clone(), merged_val);
                    }
                }
                None => {
                    merged.insert(key.clone(), value.clone());
                }
            }
        }
        ORMap { entries: merged }
    }
}

use crate::error::{SyncEngineError, SyncResult};
