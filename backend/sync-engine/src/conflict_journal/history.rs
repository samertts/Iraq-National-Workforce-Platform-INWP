use super::ConflictJournalEntry;

pub struct ConflictHistory {
    entries: Vec<ConflictJournalEntry>,
}

impl Default for ConflictHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl ConflictHistory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, entry: ConflictJournalEntry) {
        self.entries.push(entry);
    }

    pub fn timeline(&self) -> &[ConflictJournalEntry] {
        &self.entries
    }

    pub fn replay(&self, from: chrono::DateTime<chrono::Utc>) -> Vec<&ConflictJournalEntry> {
        self.entries.iter()
            .filter(|e| e.created_at >= from)
            .collect()
    }

    pub fn for_human_review(&self) -> Vec<HumanReadableConflict> {
        self.entries.iter().map(|e| HumanReadableConflict {
            conflict_id: e.conflict_id,
            record_type: e.record_type.clone(),
            record_id: e.record_id.clone(),
            local_node: e.local_node,
            remote_node: e.remote_node,
            strategy: e.resolution_strategy.clone(),
            status: if e.resolution.is_some() { "resolved" } else { "open" },
            created_at: e.created_at,
            resolved_at: e.resolved_at,
        }).collect()
    }
}

pub struct HumanReadableConflict {
    pub conflict_id: uuid::Uuid,
    pub record_type: String,
    pub record_id: String,
    pub local_node: uuid::Uuid,
    pub remote_node: uuid::Uuid,
    pub strategy: String,
    pub status: &'static str,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
}
