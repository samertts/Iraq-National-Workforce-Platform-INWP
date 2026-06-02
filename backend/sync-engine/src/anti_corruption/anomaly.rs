use super::{AnomalyType, CorruptionEvent, CorruptionSeverity};
use crate::core::types::SyncRecord;
use crate::events::contract::SyncEvent;
use crate::security::signing::SigningEngine;
use std::collections::HashSet;

pub struct AnomalyDetector {
    signing_engine: SigningEngine,
    seen_event_ids: HashSet<uuid::Uuid>,
    suspicious_threshold: u32,
}

impl AnomalyDetector {
    pub fn new(signing_engine: SigningEngine) -> Self {
        Self {
            signing_engine,
            seen_event_ids: HashSet::new(),
            suspicious_threshold: 3,
        }
    }

    pub fn inspect_event(&mut self, event: &SyncEvent) -> Option<CorruptionEvent> {
        // 1. Duplicate detection
        if !self.seen_event_ids.insert(event.event_id) {
            return Some(CorruptionEvent::new(
                AnomalyType::DuplicateEvent,
                CorruptionSeverity::Major,
                event.node_id,
                format!("Duplicate event_id: {}", event.event_id),
                serde_json::to_vec(event).unwrap_or_default(),
            ));
        }

        // 2. Signature verification
        if !event.signature.is_empty()
            && self.signing_engine.verify(&event.payload, &event.signature).is_err() {
                return Some(CorruptionEvent::new(
                    AnomalyType::InvalidSignature,
                    CorruptionSeverity::Critical,
                    event.node_id,
                    format!("Invalid signature on event {}", event.event_id),
                    event.signature.clone(),
                ));
            }

        // 3. Timestamp sanity check
        let now = chrono::Utc::now();
        let age = now - event.created_at;
        if age.num_days() > 365 {
            return Some(CorruptionEvent::new(
                AnomalyType::TimeTravel,
                CorruptionSeverity::Major,
                event.node_id,
                format!("Event timestamp is {} days in the future", age.num_days()),
                event.created_at.to_rfc3339().into_bytes(),
            ));
        }

        // 4. Schema version check
        let supported_schemas = ["1.0", "1.1", "2.0"];
        if !supported_schemas.contains(&event.schema_version.as_str()) {
            return Some(CorruptionEvent::new(
                AnomalyType::SchemaViolation,
                CorruptionSeverity::Minor,
                event.node_id,
                format!("Unknown schema version: {}", event.schema_version),
                event.schema_version.as_bytes().to_vec(),
            ));
        }

        None
    }

    pub fn inspect_record(&self, record: &SyncRecord) -> Option<CorruptionEvent> {
        // Verify record hash consistency
        let computed_hash = crate::core::merkle::compute_record_hash(
            &record.record_id,
            &record.record_type,
            &record.payload,
            record.version_vector.local_timestamp,
        );

        if !record.signature.is_empty() && record.signature != computed_hash {
            return Some(CorruptionEvent::new(
                AnomalyType::TamperedPayload,
                CorruptionSeverity::Critical,
                uuid::Uuid::nil(),
                format!("Record hash mismatch for {}", record.record_id),
                computed_hash,
            ));
        }

        None
    }

    pub fn detect_forked_history(
        &self,
        local_ancestors: &[Vec<u8>],
        remote_ancestors: &[Vec<u8>],
    ) -> Option<CorruptionEvent> {
        let common_prefix_len = local_ancestors.iter()
            .zip(remote_ancestors.iter())
            .take_while(|(a, b)| a == b)
            .count();

        if common_prefix_len < local_ancestors.len().min(remote_ancestors.len()) {
            return Some(CorruptionEvent::new(
                AnomalyType::ForkedHistory,
                CorruptionSeverity::Critical,
                uuid::Uuid::nil(),
                format!("Forked history detected at depth {}", common_prefix_len),
                Vec::new(),
            ));
        }

        None
    }

    pub fn reset(&mut self) {
        self.seen_event_ids.clear();
    }
}
