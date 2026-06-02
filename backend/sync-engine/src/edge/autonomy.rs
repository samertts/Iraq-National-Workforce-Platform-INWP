use crate::error::SyncResult;
use crate::recovery::state::RecoveryStateMachine;
use std::time::Duration;
use tracing::info;

use super::EdgeIsolationState;

pub struct EdgeAutonomyEngine {
    state: EdgeIsolationState,
    recovery_fsm: RecoveryStateMachine,
    heartbeat_missed_threshold: u32,
    reconnect_backoff: Vec<Duration>,
}

impl EdgeAutonomyEngine {
    pub fn new(
        node_id: uuid::Uuid,
        domain_id: uuid::Uuid,
        region: impl Into<String>,
    ) -> Self {
        let recovery_fsm = RecoveryStateMachine::new(node_id, region);

        Self {
            state: EdgeIsolationState {
                node_id,
                domain_id,
                autonomous_mode: false,
                disconnected_since: None,
                local_queue_depth: 0,
                last_successful_sync: None,
                corruption_quarantine_count: 0,
                pending_reconciliation_count: 0,
                local_continuity_token: uuid::Uuid::now_v7(),
                edge_capabilities: super::EdgeCapabilities {
                    max_offline_duration_days: 90,
                    local_auth_enabled: true,
                    local_workflow_enabled: true,
                    max_pending_events: 1_000_000,
                    storage_capacity_bytes: 100 * 1024 * 1024 * 1024,
                    supported_entities: vec![
                        "ClockEvent".into(),
                        "AttendanceException".into(),
                        "LeaveRequest".into(),
                        "Shift".into(),
                        "DeviceTrust".into(),
                    ],
                },
            },
            recovery_fsm,
            heartbeat_missed_threshold: 3,
            reconnect_backoff: vec![
                Duration::from_secs(1),
                Duration::from_secs(5),
                Duration::from_secs(30),
                Duration::from_secs(300),
                Duration::from_secs(1800),
            ],
        }
    }

    pub fn detect_disconnection(&mut self, missed_heartbeats: u32) -> bool {
        if missed_heartbeats >= self.heartbeat_missed_threshold && !self.state.autonomous_mode {
            self.enter_autonomous_mode()
        }
        self.state.autonomous_mode
    }

    pub fn enter_autonomous_mode(&mut self) {
        self.state.autonomous_mode = true;
        self.state.disconnected_since = Some(chrono::Utc::now());
        self.recovery_fsm.enter_autonomous_mode();
        info!(
            node_id = %self.state.node_id,
            "Edge entered autonomous mode — full local operation"
        );
    }

    pub fn attempt_reconnect(&mut self, attempt: u32) -> Duration {
        let backoff_idx = (attempt as usize).min(self.reconnect_backoff.len() - 1);
        let delay = self.reconnect_backoff[backoff_idx];
        self.recovery_fsm.record_reconnect_attempt();
        delay
    }

    pub fn on_sync_success(&mut self) {
        self.state.autonomous_mode = false;
        self.state.disconnected_since = None;
        self.state.last_successful_sync = Some(chrono::Utc::now());
        self.recovery_fsm.exit_autonomous_mode();
        info!(node_id = %self.state.node_id, "Edge reconnected — sync successful");
    }

    pub fn is_offline_too_long(&self) -> bool {
        match self.state.disconnected_since {
            Some(since) => {
                let elapsed = chrono::Utc::now() - since;
                elapsed.num_days() > self.state.edge_capabilities.max_offline_duration_days as i64
            }
            None => false,
        }
    }

    pub fn state(&self) -> &EdgeIsolationState {
        &self.state
    }

    pub fn recovery_state(&self) -> &RecoveryStateMachine {
        &self.recovery_fsm
    }

    pub fn queue_event(&mut self, _event: &crate::events::contract::SyncEvent) -> SyncResult<()> {
        self.state.local_queue_depth += 1;
        Ok(())
    }

    pub fn drain_queue(&mut self, count: u64) {
        self.state.local_queue_depth = self.state.local_queue_depth.saturating_sub(count);
    }
}
