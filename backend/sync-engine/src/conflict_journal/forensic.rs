use super::ConflictJournalEntry;
use std::collections::HashMap;

pub struct ForensicAnalyzer {
    node_conflict_patterns: HashMap<uuid::Uuid, Vec<String>>,
}

impl Default for ForensicAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ForensicAnalyzer {
    pub fn new() -> Self {
        Self {
            node_conflict_patterns: HashMap::new(),
        }
    }

    pub fn analyze_node_behavior(&self, entries: &[ConflictJournalEntry]) -> NodeBehaviorReport {
        let mut conflict_count: HashMap<uuid::Uuid, u32> = HashMap::new();
        let mut record_types: HashMap<String, u32> = HashMap::new();

        for entry in entries {
            *conflict_count.entry(entry.local_node).or_insert(0) += 1;
            *conflict_count.entry(entry.remote_node).or_insert(0) += 1;
            *record_types.entry(entry.record_type.clone()).or_insert(0) += 1;
        }

        NodeBehaviorReport {
            most_conflicted_node: conflict_count
                .iter()
                .max_by_key(|(_, c)| *c)
                .map(|(id, _)| *id),
            total_conflicts: entries.len() as u64,
            conflict_by_node: conflict_count,
            conflict_by_type: record_types,
        }
    }

    pub fn detect_conflict_clusters(
        &self,
        entries: &[ConflictJournalEntry],
    ) -> Vec<ConflictCluster> {
        let mut clusters: HashMap<String, Vec<&ConflictJournalEntry>> = HashMap::new();

        for entry in entries {
            let key = format!("{}:{}", entry.record_type, entry.record_id);
            clusters.entry(key).or_default().push(entry);
        }

        clusters
            .into_iter()
            .filter(|(_, entries)| entries.len() > 2)
            .map(|(key, entries)| {
                let conflict_count = entries.len() as u32;
                ConflictCluster {
                    cluster_key: key,
                    entries: entries.into_iter().cloned().collect(),
                    conflict_count,
                }
            })
            .collect()
    }
}

pub struct NodeBehaviorReport {
    pub most_conflicted_node: Option<uuid::Uuid>,
    pub total_conflicts: u64,
    pub conflict_by_node: HashMap<uuid::Uuid, u32>,
    pub conflict_by_type: HashMap<String, u32>,
}

pub struct ConflictCluster {
    pub cluster_key: String,
    pub entries: Vec<ConflictJournalEntry>,
    pub conflict_count: u32,
}
