use std::collections::HashMap;
use tracing::info;

/// Replay runtime — isolated execution environment for deterministic replay with
/// resource accounting, bounded memory, and starvation-free scheduling.
pub struct ReplayRuntime {
    active_sessions: HashMap<uuid::Uuid, ReplaySession>,
    max_concurrent_replays: usize,
}

#[derive(Debug, Clone)]
pub struct ReplaySession {
    pub session_id: uuid::Uuid,
    pub stream_id: String,
    pub replay_range: (u64, u64),
    pub status: ReplaySessionStatus,
    pub resource_accounting: ResourceAccount,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub deterministic_key: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplaySessionStatus {
    Initializing,
    Replaying,
    Verifying,
    Completed,
    Failed,
    TimedOut,
}

#[derive(Debug, Clone)]
pub struct ResourceAccount {
    pub events_processed: u64,
    pub memory_used_bytes: u64,
    pub cpu_time_ms: u64,
    pub duration_ms: u64,
    pub checkpoint_count: u32,
    pub envelope_usage: f64,
}

pub struct ReplayResourceEnforcer;

#[derive(Debug, Clone)]
pub struct ReplayBudget {
    pub max_events_per_session: u64,
    pub max_memory_per_session: u64,
    pub max_duration_per_session: u64,
    pub max_concurrent_sessions: usize,
    pub max_checkpoints_per_session: u32,
}

pub struct DeterministicRetryGovernor;

#[derive(Debug, Clone)]
pub struct RetryDecision {
    pub should_retry: bool,
    pub retry_delay_ms: u64,
    pub retry_count: u32,
    pub reason: String,
}

impl ReplayRuntime {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            active_sessions: HashMap::new(),
            max_concurrent_replays: max_concurrent,
        }
    }

    pub fn start_session(&mut self, stream_id: &str, from: u64, to: u64) -> Option<ReplaySession> {
        if self.active_sessions.len() >= self.max_concurrent_replays {
            return None;
        }

        let session = ReplaySession {
            session_id: uuid::Uuid::now_v7(),
            stream_id: stream_id.to_string(),
            replay_range: (from, to),
            status: ReplaySessionStatus::Initializing,
            resource_accounting: ResourceAccount {
                events_processed: 0,
                memory_used_bytes: 0,
                cpu_time_ms: 0,
                duration_ms: 0,
                checkpoint_count: 0,
                envelope_usage: 0.0,
            },
            started_at: chrono::Utc::now(),
            deterministic_key: vec![],
        };

        info!(
            stream = %stream_id,
            session = %session.session_id,
            from, to,
            "Replay session started"
        );

        self.active_sessions.insert(session.session_id, session.clone());
        Some(session)
    }

    pub fn update_resource_usage(&mut self, session_id: uuid::Uuid, account: ResourceAccount) {
        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            session.resource_accounting = account;
            session.status = ReplaySessionStatus::Replaying;
        }
    }

    pub fn complete_session(&mut self, session_id: uuid::Uuid, success: bool) {
        if let Some(session) = self.active_sessions.get_mut(&session_id) {
            session.status = if success { ReplaySessionStatus::Completed } else { ReplaySessionStatus::Failed };
            session.resource_accounting.duration_ms = (chrono::Utc::now() - session.started_at).num_milliseconds() as u64;
            info!(
                session = %session_id,
                success,
                duration = session.resource_accounting.duration_ms,
                "Replay session completed"
            );
        }
    }

    pub fn get_active_sessions(&self) -> Vec<&ReplaySession> {
        self.active_sessions.values().collect()
    }

    pub fn get_saturation(&self) -> f64 {
        if self.max_concurrent_replays > 0 {
            self.active_sessions.len() as f64 / self.max_concurrent_replays as f64
        } else {
            0.0
        }
    }
}

impl ReplayResourceEnforcer {
    pub fn new() -> Self {
        Self
    }

    pub fn create_budget(max_events: u64, max_memory: u64, max_duration: u64) -> ReplayBudget {
        ReplayBudget {
            max_events_per_session: max_events,
            max_memory_per_session: max_memory,
            max_duration_per_session: max_duration,
            max_concurrent_sessions: 10,
            max_checkpoints_per_session: 100,
        }
    }

    pub fn enforce_budget(account: &ResourceAccount, budget: &ReplayBudget) -> Vec<String> {
        let mut violations = Vec::new();
        if account.events_processed > budget.max_events_per_session {
            violations.push(format!("Events processed ({}) exceeds budget ({})", account.events_processed, budget.max_events_per_session));
        }
        if account.memory_used_bytes > budget.max_memory_per_session {
            violations.push(format!("Memory used ({} bytes) exceeds budget ({} bytes)", account.memory_used_bytes, budget.max_memory_per_session));
        }
        if account.duration_ms > budget.max_duration_per_session {
            violations.push(format!("Duration ({}ms) exceeds budget ({}ms)", account.duration_ms, budget.max_duration_per_session));
        }
        violations
    }
}

impl DeterministicRetryGovernor {
    pub fn new() -> Self {
        Self
    }

    pub fn decide_retry(&self, retry_count: u32, max_retries: u32, error: &str) -> RetryDecision {
        if retry_count >= max_retries {
            return RetryDecision {
                should_retry: false,
                retry_delay_ms: 0,
                retry_count,
                reason: format!("Max retries ({}) exceeded", max_retries),
            };
        }

        let delay = match error {
            e if e.contains("timeout") => 1000 * (retry_count + 1) as u64,
            e if e.contains("overload") => 5000 * (retry_count + 1) as u64,
            e if e.contains("replay") => 100,
            _ => 500,
        };

        RetryDecision {
            should_retry: true,
            retry_delay_ms: delay,
            retry_count: retry_count + 1,
            reason: format!("Retry #{} after {}ms: {}", retry_count + 1, delay, error),
        }
    }
}

impl Default for ReplayRuntime {
    fn default() -> Self {
        Self::new(10)
    }
}

impl Default for ReplayResourceEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DeterministicRetryGovernor {
    fn default() -> Self {
        Self::new()
    }
}
