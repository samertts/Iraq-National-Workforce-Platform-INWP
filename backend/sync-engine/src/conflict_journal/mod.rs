pub mod forensic;
pub mod history;

pub use forensic::*;
pub use history::*;

use serde::{Deserialize, Serialize};

/// A recorded conflict with full forensic context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictJournalEntry {
    pub entry_id: uuid::Uuid,
    pub conflict_id: uuid::Uuid,
    pub partition_key: String,
    pub record_id: String,
    pub record_type: String,
    pub local_version: serde_json::Value,
    pub remote_version: serde_json::Value,
    pub local_node: uuid::Uuid,
    pub remote_node: uuid::Uuid,
    pub resolution_strategy: String,
    pub resolution: Option<String>,
    pub resolved_by: Option<uuid::Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub evidence_chain: Vec<uuid::Uuid>,
}

impl ConflictJournalEntry {
    pub fn new(
        conflict_id: uuid::Uuid,
        partition_key: impl Into<String>,
        record_id: impl Into<String>,
        record_type: impl Into<String>,
        local_node: uuid::Uuid,
        remote_node: uuid::Uuid,
        strategy: impl Into<String>,
    ) -> Self {
        Self {
            entry_id: uuid::Uuid::now_v7(),
            conflict_id,
            partition_key: partition_key.into(),
            record_id: record_id.into(),
            record_type: record_type.into(),
            local_version: serde_json::Value::Null,
            remote_version: serde_json::Value::Null,
            local_node,
            remote_node,
            resolution_strategy: strategy.into(),
            resolution: None,
            resolved_by: None,
            created_at: chrono::Utc::now(),
            resolved_at: None,
            evidence_chain: Vec::new(),
        }
    }
}

/// A complete conflict journal for forensic analysis
pub struct ConflictJournal {
    entries: Vec<ConflictJournalEntry>,
    max_entries: usize,
}

impl ConflictJournal {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    pub fn record(&mut self, entry: ConflictJournalEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[ConflictJournalEntry] {
        &self.entries
    }

    pub fn entries_for_record(&self, record_id: &str) -> Vec<&ConflictJournalEntry> {
        self.entries
            .iter()
            .filter(|e| e.record_id == record_id)
            .collect()
    }

    pub fn entries_for_node(&self, node_id: &uuid::Uuid) -> Vec<&ConflictJournalEntry> {
        self.entries
            .iter()
            .filter(|e| e.local_node == *node_id || e.remote_node == *node_id)
            .collect()
    }

    pub fn unresolved(&self) -> Vec<&ConflictJournalEntry> {
        self.entries
            .iter()
            .filter(|e| e.resolution.is_none())
            .collect()
    }

    pub fn forensic_trail(&self, entry_id: &uuid::Uuid) -> Option<ForensicTrail> {
        let entry = self.entries.iter().find(|e| e.entry_id == *entry_id)?;
        Some(ForensicTrail {
            entry: entry.clone(),
            timeline: self
                .entries
                .iter()
                .filter(|e| e.record_id == entry.record_id)
                .cloned()
                .collect(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ForensicTrail {
    pub entry: ConflictJournalEntry,
    pub timeline: Vec<ConflictJournalEntry>,
}
