use serde::{Deserialize, Serialize};
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryPhase {
    Detection,
    Isolation,
    Reconnection,
    Catchup,
    Normalization,
    Verification,
    Complete,
    Failed,
}

impl RecoveryPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Detection => "detection",
            Self::Isolation => "isolation",
            Self::Reconnection => "reconnection",
            Self::Catchup => "catchup",
            Self::Normalization => "normalization",
            Self::Verification => "verification",
            Self::Complete => "complete",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryStateMachine {
    pub node_id: uuid::Uuid,
    pub region: String,
    pub current_phase: RecoveryPhase,
    pub last_completed_phase: Option<RecoveryPhase>,
    pub autonomous_mode: bool,
    pub pending_events: u64,
    pub synced_events: u64,
    pub conflict_count: u32,
    pub last_reconnect_attempt: Option<chrono::DateTime<chrono::Utc>>,
    pub reconnect_attempts: u32,
    pub last_error: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

impl RecoveryStateMachine {
    pub fn new(node_id: uuid::Uuid, region: impl Into<String>) -> Self {
        Self {
            node_id,
            region: region.into(),
            current_phase: RecoveryPhase::Detection,
            last_completed_phase: None,
            autonomous_mode: false,
            pending_events: 0,
            synced_events: 0,
            conflict_count: 0,
            last_reconnect_attempt: None,
            reconnect_attempts: 0,
            last_error: None,
            started_at: chrono::Utc::now(),
        }
    }

    pub fn transition_to(&mut self, phase: RecoveryPhase) -> SyncResult<()> {
        if !self.is_valid_transition(phase) {
            return Err(SyncEngineError::Recovery(format!(
                "Invalid phase transition: {:?} -> {:?}",
                self.current_phase, phase
            )));
        }

        info!(
            from = %self.current_phase.as_str(),
            to = %phase.as_str(),
            "Recovery state transition"
        );

        self.last_completed_phase = Some(self.current_phase);
        self.current_phase = phase;
        Ok(())
    }

    pub fn enter_autonomous_mode(&mut self) {
        self.autonomous_mode = true;
        info!("Entering autonomous mode");
    }

    pub fn exit_autonomous_mode(&mut self) {
        self.autonomous_mode = false;
        info!("Exiting autonomous mode");
    }

    pub fn record_reconnect_attempt(&mut self) {
        self.reconnect_attempts += 1;
        self.last_reconnect_attempt = Some(chrono::Utc::now());
    }

    pub fn record_progress(&mut self, events_synced: u64) {
        self.synced_events += events_synced;
        self.pending_events = self.pending_events.saturating_sub(events_synced);
    }

    pub fn record_conflict(&mut self) {
        self.conflict_count += 1;
    }

    pub fn set_error(&mut self, error: impl Into<String>) {
        self.last_error = Some(error.into());
        warn!(
            phase = %self.current_phase.as_str(),
            error = %self.last_error.as_ref().unwrap(),
            "Recovery error"
        );
    }

    pub fn elapsed(&self) -> chrono::Duration {
        chrono::Utc::now() - self.started_at
    }

    pub fn is_complete(&self) -> bool {
        self.current_phase == RecoveryPhase::Complete
    }

    pub fn is_failed(&self) -> bool {
        self.current_phase == RecoveryPhase::Failed
    }

    pub fn needs_autonomous_operation(&self, missed_heartbeats: u32, threshold: u32) -> bool {
        missed_heartbeats >= threshold && !self.autonomous_mode
    }

    fn is_valid_transition(&self, target: RecoveryPhase) -> bool {
        use RecoveryPhase::*;
        matches!(
            (self.current_phase, target),
            (Detection, Isolation)
                | (Isolation, Reconnection)
                | (Reconnection, Catchup)
                | (Catchup, Normalization)
                | (Normalization, Verification)
                | (Verification, Complete)
                | (_, Failed)
                | (Failed, Detection)
        )
    }
}

use crate::error::{SyncEngineError, SyncResult};
