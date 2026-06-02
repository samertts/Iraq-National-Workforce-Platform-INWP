use crate::events::contract::SyncEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event: SyncEvent,
    pub routing_key: String,
    pub delivery_attempt: u32,
    pub published_at: chrono::DateTime<chrono::Utc>,
    pub ttl_seconds: u64,
}

impl EventEnvelope {
    pub fn new(event: SyncEvent, routing_key: impl Into<String>) -> Self {
        Self {
            event,
            routing_key: routing_key.into(),
            delivery_attempt: 1,
            published_at: chrono::Utc::now(),
            ttl_seconds: 3600,
        }
    }

    pub fn with_ttl(mut self, ttl_seconds: u64) -> Self {
        self.ttl_seconds = ttl_seconds;
        self
    }

    pub fn is_expired(&self) -> bool {
        let elapsed = chrono::Utc::now() - self.published_at;
        elapsed.num_seconds() > self.ttl_seconds as i64
    }

    pub fn increment_attempt(&mut self) {
        self.delivery_attempt += 1;
    }
}
