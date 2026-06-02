use super::DottedVersionVector;

pub struct ClockReconciler {
    local_clock: DottedVersionVector,
    max_clock_drift_seconds: i64,
}

impl ClockReconciler {
    pub fn new(node_id: uuid::Uuid, max_clock_drift_seconds: i64) -> Self {
        Self {
            local_clock: DottedVersionVector::new(node_id),
            max_clock_drift_seconds,
        }
    }

    pub fn reconcile(&mut self, remote_clock: &DottedVersionVector) -> ReconciliationOutcome {
        if self.local_clock.concurrent(remote_clock) {
            // Both sides have diverged — need conflict resolution
            self.local_clock.merge(remote_clock);
            ReconciliationOutcome::ConcurrentMerge
        } else if remote_clock.happens_before(&self.local_clock) {
            // We are ahead — no action needed
            ReconciliationOutcome::LocalAhead
        } else if self.local_clock.happens_before(remote_clock) {
            // They are ahead — catch up
            self.local_clock.merge(remote_clock);
            ReconciliationOutcome::RemoteAhead
        } else {
            ReconciliationOutcome::Identical
        }
    }

    pub fn detect_clock_drift(
        &self,
        remote_timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Option<chrono::Duration> {
        let drift = chrono::Utc::now() - remote_timestamp;
        if drift.num_seconds().abs() > self.max_clock_drift_seconds {
            Some(drift)
        } else {
            None
        }
    }

    pub fn local_clock(&self) -> &DottedVersionVector {
        &self.local_clock
    }

    pub fn local_clock_mut(&mut self) -> &mut DottedVersionVector {
        &mut self.local_clock
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconciliationOutcome {
    LocalAhead,
    RemoteAhead,
    ConcurrentMerge,
    Identical,
}
