use super::CorruptionEvent;
use std::collections::VecDeque;
use tracing::{info, warn};

pub struct CorruptionQuarantine {
    quarantined_events: VecDeque<CorruptionEvent>,
    max_quarantine_size: usize,
    auto_resolve_after_hours: u64,
}

impl CorruptionQuarantine {
    pub fn new(max_quarantine_size: usize, auto_resolve_after_hours: u64) -> Self {
        Self {
            quarantined_events: VecDeque::new(),
            max_quarantine_size,
            auto_resolve_after_hours,
        }
    }

    pub fn quarantine(&mut self, event: CorruptionEvent) -> bool {
        if self.quarantined_events.len() >= self.max_quarantine_size {
            warn!("Quarantine full — discarding oldest corruption event");
            self.quarantined_events.pop_front();
        }

        info!(
            anomaly_type = %event.anomaly_type.as_str(),
            severity = ?event.severity,
            source = %event.source_node,
            "Event quarantined for investigation"
        );

        self.quarantined_events.push_back(event);
        true
    }

    pub fn resolve(&mut self, event_id: &uuid::Uuid) -> Option<CorruptionEvent> {
        if let Some(pos) = self.quarantined_events.iter().position(|e| e.event_id == *event_id) {
            let mut event = self.quarantined_events.remove(pos)?;
            event.resolved = true;
            info!(event_id = %event_id, "Corruption event resolved");
            Some(event)
        } else {
            None
        }
    }

    pub fn quarantine_depth(&self) -> usize {
        self.quarantined_events.len()
    }

    pub fn pending_investigations(&self) -> Vec<&CorruptionEvent> {
        self.quarantined_events.iter()
            .filter(|e| !e.resolved)
            .collect()
    }

    pub fn auto_resolve_stale(&mut self) -> u32 {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(self.auto_resolve_after_hours as i64);
        let mut resolved = 0;

        self.quarantined_events.retain(|e| {
            if e.detected_at < cutoff && !e.resolved {
                resolved += 1;
                false
            } else {
                true
            }
        });

        if resolved > 0 {
            info!(count = resolved, "Auto-resolved stale quarantine events");
        }

        resolved
    }

    pub fn events_for_replay(&self) -> Vec<&CorruptionEvent> {
        self.quarantined_events.iter().collect()
    }
}
