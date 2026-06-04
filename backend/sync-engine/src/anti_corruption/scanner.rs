use super::{AnomalyType, CorruptionEvent, CorruptionSeverity};
use crate::core::node::NodeIdentity;
use crate::core::types::SyncRecord;
use std::collections::HashMap;
use tracing::warn;

pub struct CorruptionScanner {
    node_trust_scores: HashMap<uuid::Uuid, f64>,
    anomaly_counts: HashMap<uuid::Uuid, u32>,
    scanner_id: uuid::Uuid,
}

impl Default for CorruptionScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl CorruptionScanner {
    pub fn new() -> Self {
        Self {
            node_trust_scores: HashMap::new(),
            anomaly_counts: HashMap::new(),
            scanner_id: uuid::Uuid::now_v7(),
        }
    }

    pub fn scan_batch(
        &mut self,
        records: &[SyncRecord],
        source: &NodeIdentity,
    ) -> Vec<CorruptionEvent> {
        let mut events = Vec::new();

        for record in records {
            // Check for duplicate record_ids within batch
            let duplicate_count = records
                .iter()
                .filter(|r| r.record_id == record.record_id)
                .count();
            if duplicate_count > 1 {
                events.push(CorruptionEvent::new(
                    AnomalyType::DuplicateEvent,
                    CorruptionSeverity::Minor,
                    source.node_id,
                    format!("Duplicate record in batch: {}", record.record_id),
                    record.record_id.as_bytes().to_vec(),
                ));
            }

            // Check payload size anomalies
            if record.payload.len() > 10_000_000 {
                events.push(CorruptionEvent::new(
                    AnomalyType::SchemaViolation,
                    CorruptionSeverity::Major,
                    source.node_id,
                    format!("Suspiciously large payload: {} bytes", record.payload.len()),
                    record.payload[..100].to_vec(),
                ));
            }
        }

        // Track per-node anomaly counts
        if !events.is_empty() {
            let count = self.anomaly_counts.entry(source.node_id).or_insert(0);
            *count += events.len() as u32;
            self.update_trust_score(source.node_id);
        }

        events
    }

    pub fn scan_node_activity(&mut self, node: &NodeIdentity) -> Option<CorruptionEvent> {
        let anomaly_count = self.anomaly_counts.get(&node.node_id).copied().unwrap_or(0);

        if anomaly_count > 10 {
            return Some(CorruptionEvent::new(
                AnomalyType::RogueNodeActivity,
                CorruptionSeverity::Critical,
                node.node_id,
                format!("Node has {} anomalies — possible compromise", anomaly_count),
                node.public_key.clone(),
            ));
        }

        None
    }

    pub fn trust_score(&self, node_id: &uuid::Uuid) -> f64 {
        self.node_trust_scores.get(node_id).copied().unwrap_or(1.0)
    }

    pub fn reset_node(&mut self, node_id: &uuid::Uuid) {
        self.anomaly_counts.remove(node_id);
        self.node_trust_scores.remove(node_id);
    }

    fn update_trust_score(&mut self, node_id: uuid::Uuid) {
        let anomaly_count = self.anomaly_counts.get(&node_id).copied().unwrap_or(0);
        let score = 1.0 / (1.0 + anomaly_count as f64);
        self.node_trust_scores.insert(node_id, score);

        if score < 0.5 {
            warn!(node_id = %node_id, trust_score = score, "Node trust score degraded");
        }
    }
}
