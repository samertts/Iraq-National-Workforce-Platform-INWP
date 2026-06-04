use sha2::Digest;
use std::collections::{HashMap, VecDeque};
use std::panic::{self, AssertUnwindSafe};
use tracing::{info, warn};

/// Deterministic async scheduler — governs all async execution with bounded concurrency,
/// deterministic ordering, starvation prevention, and overload isolation.
pub struct DeterministicScheduler {
    max_concurrency: usize,
    execution_queue: VecDeque<ScheduledTask>,
    active_tasks: HashMap<uuid::Uuid, TaskState>,
    scheduler_id: uuid::Uuid,
    deterministic_seed: u64,
}

#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub task_id: uuid::Uuid,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub max_duration_ms: u64,
    pub memory_budget_bytes: u64,
    pub deterministic_key: Option<String>,
    pub retry_policy: RetryPolicy,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub age_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskType {
    SyncOperation,
    ReplayOperation,
    Reconciliation,
    EventProcessing,
    CheckpointCreation,
    FederationSync,
    GovernanceEvaluation,
    Compaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Background,
    Normal,
    High,
    Critical,
    Sovereign,
}

#[derive(Debug, Clone)]
pub struct TaskState {
    pub task_id: uuid::Uuid,
    pub status: ExecutionStatus,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub retry_count: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Queued,
    Running,
    Completed,
    Failed,
    TimedOut,
    Cancelled,
}

#[derive(Debug, Clone)]
pub enum RetryPolicy {
    NoRetry,
    FixedDelay(u32, u64),
    ExponentialBackoff(u32, u64, u64),
}

pub struct ExecutionEnvelopeEngine;

#[derive(Debug, Clone)]
pub struct ExecutionEnvelope {
    pub envelope_id: uuid::Uuid,
    pub task_type: TaskType,
    pub max_duration_ms: u64,
    pub max_memory_bytes: u64,
    pub deterministic: bool,
    pub replay_safe: bool,
    pub isolation_group: String,
}

pub struct OverloadController {
    pub controller_id: uuid::Uuid,
    pub current_load: f64,
    pub overload_threshold: f64,
    pub is_overloaded: bool,
    pub degraded_since: Option<chrono::DateTime<chrono::Utc>>,
    pub rejection_count: u64,
}

pub struct BoundedWorkerRuntime;

#[derive(Debug)]
pub struct WorkerPoolState {
    pub pool_id: uuid::Uuid,
    pub max_workers: usize,
    pub active_workers: usize,
    pub queue_depth: usize,
    pub saturation: f64,
}

pub struct DeterministicTimeoutEngine;

impl DeterministicScheduler {
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            max_concurrency,
            execution_queue: VecDeque::new(),
            active_tasks: HashMap::new(),
            scheduler_id: uuid::Uuid::now_v7(),
            deterministic_seed: 42,
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.deterministic_seed = seed;
        self
    }

    pub fn submit(&mut self, task: ScheduledTask) {
        let state = TaskState {
            task_id: task.task_id,
            status: ExecutionStatus::Queued,
            started_at: None,
            retry_count: 0,
            error: None,
        };
        self.active_tasks.insert(task.task_id, state);
        self.execution_queue.push_back(task);
    }

    pub fn schedule_next(&mut self) -> Option<ScheduledTask> {
        if self
            .active_tasks
            .values()
            .filter(|s| matches!(s.status, ExecutionStatus::Running))
            .count()
            >= self.max_concurrency
        {
            return None;
        }

        // Enforce timeouts first — expire tasks that have exceeded max_duration
        self.enforce_timeouts();

        // Age all queued tasks to prevent starvation
        for task in &mut self.execution_queue {
            if let Some(state) = self.active_tasks.get(&task.task_id) {
                if matches!(state.status, ExecutionStatus::Queued) {
                    task.age_count = task.age_count.saturating_add(1);
                }
            }
        }

        let mut best_idx = None;
        let mut best_score: i64 = i64::MIN;

        for (i, task) in self.execution_queue.iter().enumerate() {
            if !self.active_tasks.contains_key(&task.task_id) {
                continue;
            }
            let state = self.active_tasks.get(&task.task_id).unwrap();
            if !matches!(state.status, ExecutionStatus::Queued) {
                continue;
            }

            // Combine base priority, age bonus (anti-starvation), and deterministic tiebreaker
            let priority_score = priority_to_score(task.priority);
            let age_bonus = (task.age_count as i64).min(1000);
            let det_tiebreaker = self.deterministic_tiebreaker(&task.task_id);
            let score = priority_score + age_bonus + det_tiebreaker;

            if score > best_score {
                best_score = score;
                best_idx = Some(i);
            }
        }

        best_idx.map(|idx| self.execution_queue.remove(idx).unwrap())
    }

    /// Deterministic tiebreaker based on task_id and seed — ensures same-priority
    /// tasks are ordered reproducibly across replays.
    fn deterministic_tiebreaker(&self, task_id: &uuid::Uuid) -> i64 {
        let hash = sha2::Sha256::new()
            .chain_update(self.deterministic_seed.to_le_bytes())
            .chain_update(task_id.as_bytes())
            .finalize();
        let val = u64::from_le_bytes(hash[..8].try_into().unwrap_or([0u8; 8]));
        (val % 1000) as i64
    }

    /// Cancel a queued or running task by ID
    pub fn cancel_task(&mut self, task_id: uuid::Uuid) -> bool {
        let before = self.execution_queue.len();
        self.execution_queue.retain(|t| t.task_id != task_id);
        let removed = before > self.execution_queue.len();
        let canceled = if let Some(state) = self.active_tasks.get_mut(&task_id) {
            if matches!(
                state.status,
                ExecutionStatus::Queued | ExecutionStatus::Running
            ) {
                state.status = ExecutionStatus::Cancelled;
                true
            } else {
                false
            }
        } else {
            false
        };
        removed || canceled
    }

    /// Enforce timeouts — mark tasks that have exceeded max_duration as TimedOut
    fn enforce_timeouts(&mut self) {
        let now = chrono::Utc::now();
        let to_remove: Vec<uuid::Uuid> = self
            .execution_queue
            .iter()
            .filter_map(|task| {
                let elapsed = (now - task.submitted_at).num_milliseconds() as u64;
                if elapsed > task.max_duration_ms {
                    self.active_tasks.get(&task.task_id).and_then(|state| {
                        if matches!(
                            state.status,
                            ExecutionStatus::Queued | ExecutionStatus::Running
                        ) {
                            Some(task.task_id)
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            })
            .collect();

        if to_remove.is_empty() {
            return;
        }

        for tid in &to_remove {
            if let Some(state) = self.active_tasks.get_mut(tid) {
                state.status = ExecutionStatus::TimedOut;
            }
        }
        self.execution_queue
            .retain(|t| !to_remove.contains(&t.task_id));

        warn!(
            count = to_remove.len(),
            "Tasks timed out and removed from queue"
        );
    }

    pub fn complete_task(&mut self, task_id: uuid::Uuid, success: bool) {
        if let Some(state) = self.active_tasks.get_mut(&task_id) {
            state.status = if success {
                ExecutionStatus::Completed
            } else {
                ExecutionStatus::Failed
            };
        }
    }

    pub fn get_state(&self) -> SchedulerReport {
        SchedulerReport {
            scheduler_id: self.scheduler_id,
            queue_depth: self.execution_queue.len(),
            active_count: self
                .active_tasks
                .values()
                .filter(|s| matches!(s.status, ExecutionStatus::Running))
                .count(),
            max_concurrency: self.max_concurrency,
            saturation: if self.max_concurrency > 0 {
                self.active_tasks
                    .values()
                    .filter(|s| matches!(s.status, ExecutionStatus::Running))
                    .count() as f64
                    / self.max_concurrency as f64
            } else {
                0.0
            },
        }
    }

    pub fn timed_out_count(&self) -> usize {
        self.active_tasks
            .values()
            .filter(|s| matches!(s.status, ExecutionStatus::TimedOut))
            .count()
    }

    pub fn cancelled_count(&self) -> usize {
        self.active_tasks
            .values()
            .filter(|s| matches!(s.status, ExecutionStatus::Cancelled))
            .count()
    }
}

const fn priority_to_score(p: TaskPriority) -> i64 {
    match p {
        TaskPriority::Background => 0,
        TaskPriority::Normal => 1000,
        TaskPriority::High => 2000,
        TaskPriority::Critical => 3000,
        TaskPriority::Sovereign => 4000,
    }
}

/// Panic containment engine — wraps task execution in panic::catch_unwind to
/// prevent a single panicked task from bringing down the entire runtime.
pub struct PanicContainmentEngine;

impl PanicContainmentEngine {
    pub fn new() -> Self {
        Self
    }

    /// Execute a closure with panic containment. Returns Ok(T) on success,
    /// or Err(String) with the panic message if the closure panics.
    pub fn execute<F, T>(&self, task_name: &str, f: F) -> Result<T, String>
    where
        F: FnOnce() -> T + std::panic::UnwindSafe,
    {
        match panic::catch_unwind(AssertUnwindSafe(f)) {
            Ok(result) => Ok(result),
            Err(payload) => {
                let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = payload.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };
                warn!(task = %task_name, panic = %msg, "Panic contained in task execution");
                Err(msg)
            }
        }
    }

    /// Execute with a fallback value on panic.
    pub fn execute_or<F, T>(&self, task_name: &str, f: F, fallback: T) -> T
    where
        F: FnOnce() -> T + std::panic::UnwindSafe,
        T: Clone,
    {
        self.execute(task_name, f).unwrap_or_else(|_| {
            warn!(task = %task_name, "Using fallback value after panic");
            fallback
        })
    }
}

impl Default for PanicContainmentEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct SchedulerReport {
    pub scheduler_id: uuid::Uuid,
    pub queue_depth: usize,
    pub active_count: usize,
    pub max_concurrency: usize,
    pub saturation: f64,
}

impl ExecutionEnvelopeEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn create_envelope(
        &self,
        task_type: TaskType,
        duration_ms: u64,
        memory_bytes: u64,
    ) -> ExecutionEnvelope {
        ExecutionEnvelope {
            envelope_id: uuid::Uuid::now_v7(),
            task_type,
            max_duration_ms: duration_ms,
            max_memory_bytes: memory_bytes,
            deterministic: true,
            replay_safe: true,
            isolation_group: format!("{:?}", task_type),
        }
    }
}

impl OverloadController {
    pub fn new(threshold: f64) -> Self {
        Self {
            controller_id: uuid::Uuid::now_v7(),
            current_load: 0.0,
            overload_threshold: threshold,
            is_overloaded: false,
            degraded_since: None,
            rejection_count: 0,
        }
    }

    pub fn assess_load(&mut self, active_tasks: usize, max_tasks: usize) -> f64 {
        self.current_load = if max_tasks > 0 {
            active_tasks as f64 / max_tasks as f64
        } else {
            0.0
        };

        let was_overloaded = self.is_overloaded;
        self.is_overloaded = self.current_load >= self.overload_threshold;

        if self.is_overloaded && !was_overloaded {
            self.degraded_since = Some(chrono::Utc::now());
            info!(
                load = self.current_load,
                threshold = self.overload_threshold,
                "Overload condition detected — degradation engaged"
            );
        } else if !self.is_overloaded && was_overloaded {
            self.degraded_since = None;
            info!("Overload cleared — normal operation resumed");
        }

        self.current_load
    }

    pub fn should_reject(&self) -> bool {
        self.is_overloaded
    }
}

impl BoundedWorkerRuntime {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_pool_state(active: usize, max: usize, queue: usize) -> WorkerPoolState {
        WorkerPoolState {
            pool_id: uuid::Uuid::now_v7(),
            max_workers: max,
            active_workers: active,
            queue_depth: queue,
            saturation: if max > 0 {
                active as f64 / max as f64
            } else {
                0.0
            },
        }
    }
}

impl DeterministicTimeoutEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_timeout(
        &self,
        task_type: TaskType,
        estimated_duration_ms: u64,
        retry_count: u32,
    ) -> u64 {
        let base = match task_type {
            TaskType::SyncOperation => 30_000,
            TaskType::ReplayOperation => 300_000,
            TaskType::Reconciliation => 60_000,
            TaskType::EventProcessing => 5_000,
            TaskType::CheckpointCreation => 30_000,
            TaskType::FederationSync => 120_000,
            TaskType::GovernanceEvaluation => 10_000,
            TaskType::Compaction => 300_000,
        };
        base.max(estimated_duration_ms) * (1 + retry_count).min(5) as u64
    }
}

impl Default for DeterministicScheduler {
    fn default() -> Self {
        Self::new(64)
    }
}

impl Default for ExecutionEnvelopeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OverloadController {
    fn default() -> Self {
        Self::new(0.8)
    }
}

impl Default for BoundedWorkerRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DeterministicTimeoutEngine {
    fn default() -> Self {
        Self::new()
    }
}
