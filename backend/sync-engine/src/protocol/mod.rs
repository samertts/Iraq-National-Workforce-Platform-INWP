pub mod codec;
pub mod commit;
pub mod discovery;
pub mod exchange;
pub mod handshake;
pub mod reconcile;
pub mod transfer;

pub use codec::*;
pub use commit::*;
pub use discovery::*;
pub use exchange::*;
pub use handshake::*;
pub use reconcile::*;
pub use transfer::*;

use crate::core::{SyncPhase, SyncSessionId};
use std::time::Instant;

#[derive(Debug)]
pub struct SyncSession {
    pub session_id: SyncSessionId,
    pub initiator: uuid::Uuid,
    pub target: uuid::Uuid,
    pub current_phase: SyncPhase,
    pub partitions: Vec<String>,
    pub started_at: Instant,
    pub last_activity: Instant,
    pub bytes_transferred: u64,
    pub records_synced: u32,
}

impl SyncSession {
    pub fn new(initiator: uuid::Uuid, target: uuid::Uuid, partitions: Vec<String>) -> Self {
        Self {
            session_id: uuid::Uuid::now_v7(),
            initiator,
            target,
            current_phase: SyncPhase::Discovery,
            partitions,
            started_at: Instant::now(),
            last_activity: Instant::now(),
            bytes_transferred: 0,
            records_synced: 0,
        }
    }

    pub fn advance_phase(&mut self, phase: SyncPhase) {
        self.current_phase = phase;
        self.last_activity = Instant::now();
    }

    pub fn add_transfer(&mut self, bytes: u64, records: u32) {
        self.bytes_transferred += bytes;
        self.records_synced += records;
        self.last_activity = Instant::now();
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.started_at.elapsed()
    }

    pub fn idle_for(&self) -> std::time::Duration {
        self.last_activity.elapsed()
    }
}
