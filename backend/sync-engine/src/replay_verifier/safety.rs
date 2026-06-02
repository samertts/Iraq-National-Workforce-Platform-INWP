use crate::error::{SyncEngineError, SyncResult};
use crate::events::contract::SyncEvent;
use crate::event_store::chain::EventChain;
use tracing::info;

pub struct ReplaySafetyGuard {
    max_replay_attempts: u32,
    replay_attempts: std::collections::HashMap<String, u32>,
    safety_token: uuid::Uuid,
}

impl ReplaySafetyGuard {
    pub fn new(max_replay_attempts: u32) -> Self {
        Self {
            max_replay_attempts,
            replay_attempts: std::collections::HashMap::new(),
            safety_token: uuid::Uuid::now_v7(),
        }
    }

    pub fn authorize_replay(&mut self, partition_key: &str) -> SyncResult<ReplayAuthorization> {
        let attempts = self.replay_attempts.entry(partition_key.to_string()).or_insert(0);
        *attempts += 1;

        if *attempts > self.max_replay_attempts {
            return Err(SyncEngineError::Recovery(format!(
                "Replay blocked: partition {} exceeded max attempts ({})",
                partition_key, self.max_replay_attempts
            )));
        }

        info!(
            partition = %partition_key,
            attempt = *attempts,
            "Replay authorized"
        );

        Ok(ReplayAuthorization {
            token: uuid::Uuid::now_v7(),
            partition_key: partition_key.to_string(),
            attempt_number: *attempts,
            authorized_at: chrono::Utc::now(),
        })
    }

    pub fn verify_replay_safety(
        &self,
        chain: &EventChain,
        events: &[SyncEvent],
    ) -> SyncResult<()> {
        // 1. Chain must not be sealed during replay
        if chain.sealed {
            return Err(SyncEngineError::Recovery(
                "Cannot replay into a sealed chain".into(),
            ));
        }

        // 2. Events must be in chronological order
        for window in events.windows(2) {
            if window[0].created_at > window[1].created_at {
                return Err(SyncEngineError::Recovery(format!(
                    "Replay safety violation: events out of order ({} before {})",
                    window[1].event_id, window[0].event_id
                )));
            }
        }

        // 3. No duplicate event_ids
        let mut seen = std::collections::HashSet::new();
        for event in events {
            if !seen.insert(event.event_id) {
                return Err(SyncEngineError::Recovery(format!(
                    "Replay safety violation: duplicate event {}",
                    event.event_id
                )));
            }
        }

        // 4. Chain depth consistency
        for event in events {
            if chain.chain.iter().any(|ce| ce.event_id == event.event_id) {
                return Err(SyncEngineError::Recovery(format!(
                    "Replay safety violation: event {} already in chain",
                    event.event_id
                )));
            }
        }

        Ok(())
    }

    pub fn reset_attempts(&mut self, partition_key: &str) {
        self.replay_attempts.remove(partition_key);
    }
}

#[derive(Debug)]
pub struct ReplayAuthorization {
    pub token: uuid::Uuid,
    pub partition_key: String,
    pub attempt_number: u32,
    pub authorized_at: chrono::DateTime<chrono::Utc>,
}
