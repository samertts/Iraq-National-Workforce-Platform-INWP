
pub struct EventFilter {
    allowed_event_types: Vec<String>,
    allowed_partitions: Vec<String>,
}

impl EventFilter {
    pub fn new(allowed_event_types: Vec<String>, allowed_partitions: Vec<String>) -> Self {
        Self {
            allowed_event_types,
            allowed_partitions,
        }
    }

    pub fn should_accept(&self, event_type: &str, partition_key: &str) -> bool {
        let type_match = self.allowed_event_types.is_empty()
            || self.allowed_event_types.iter().any(|t| t == event_type);
        let partition_match = self.allowed_partitions.is_empty()
            || self.allowed_partitions.iter().any(|p| partition_key.starts_with(p));

        type_match && partition_match
    }

    pub fn filter_events(
        &self,
        events: Vec<crate::events::contract::SyncEvent>,
    ) -> Vec<crate::events::contract::SyncEvent> {
        events
            .into_iter()
            .filter(|e| self.should_accept(&e.event_type, &e.partition_key))
            .collect()
    }
}
