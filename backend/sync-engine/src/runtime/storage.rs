use sha2::Digest;
use std::collections::VecDeque;
use tracing::{info, warn};

/// Storage governance — append-only enforcement, compaction governance, snapshot management,
/// quota enforcement, and immutable retention for the sovereign runtime.
pub struct StorageGovernor;

#[derive(Debug, Clone)]
pub struct StoragePolicy {
    pub policy_id: uuid::Uuid,
    pub policy_type: StoragePolicyType,
    pub retention_days: u64,
    pub max_size_bytes: u64,
    pub compaction_enabled: bool,
    pub snapshot_interval_secs: u64,
    pub encryption_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoragePolicyType {
    EventStore,
    CheckpointStore,
    AuditLog,
    ConflictJournal,
    ReplayArchive,
    LocalCache,
}

pub struct SnapshotEngine;

#[derive(Debug)]
pub struct SnapshotManifest {
    pub snapshot_id: uuid::Uuid,
    pub domain: String,
    pub stream_id: String,
    pub event_count: u64,
    pub snapshot_hash: Vec<u8>,
    pub checkpoint_depth: u64,
    pub taken_at: chrono::DateTime<chrono::Utc>,
    pub deterministic: bool,
}

pub struct CompactionEngine;

#[derive(Debug)]
pub struct CompactionPlan {
    pub plan_id: uuid::Uuid,
    pub target: String,
    pub compaction_type: CompactionType,
    pub estimated_space_savings: u64,
    pub replay_safe: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactionType {
    CrdtMerge,
    SnapshotPruning,
    LogCompaction,
    IndexRebuild,
}

pub struct QuotaEnforcer;

#[derive(Debug)]
pub struct QuotaState {
    pub domain: String,
    pub storage_used_bytes: u64,
    pub storage_quota_bytes: u64,
    pub usage_ratio: f64,
    pub over_quota: bool,
    pub enforcement_action: QuotaAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaAction {
    None,
    Warn,
    Throttle,
    BlockWrites,
    EmergencyCompaction,
}

pub struct ReplayRetentionEngine;

#[derive(Debug)]
pub struct RetentionPlan {
    pub plan_id: uuid::Uuid,
    pub stream_id: String,
    pub retain_from: chrono::DateTime<chrono::Utc>,
    pub retain_until: chrono::DateTime<chrono::Utc>,
    pub pruning_strategy: PruningStrategy,
    pub archive_target: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PruningStrategy {
    NoPruning,
    AgeBased(u64),
    SizeBased(u64),
    CheckpointBased(u64),
    GovernanceApproved,
}

/// Storage pressure analysis — tracks trends, predicts exhaustion, and recommends action.
pub struct StoragePressureEngine {
    history: VecDeque<PressureSample>,
    max_samples: usize,
}

#[derive(Debug, Clone)]
pub struct PressureSample {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub usage_ratio: f64,
    pub growth_rate_per_sec: f64,
}

#[derive(Debug)]
pub struct PressureReport {
    pub current_ratio: f64,
    pub trend: PressureTrend,
    pub estimated_exhaustion_hours: Option<f64>,
    pub growth_rate_per_hour: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureTrend {
    Declining,
    Stable,
    Growing,
    Critical,
}

impl StorageGovernor {
    pub fn new() -> Self {
        Self
    }

    pub fn create_policy(
        policy_type: StoragePolicyType,
        retention_days: u64,
        max_size: u64,
    ) -> StoragePolicy {
        StoragePolicy {
            policy_id: uuid::Uuid::now_v7(),
            policy_type,
            retention_days,
            max_size_bytes: max_size,
            compaction_enabled: true,
            snapshot_interval_secs: 3600,
            encryption_required: matches!(
                policy_type,
                StoragePolicyType::AuditLog | StoragePolicyType::ConflictJournal
            ),
        }
    }

    pub fn verify_append_only(&self, log_hashes: &[Vec<u8>]) -> bool {
        for window in log_hashes.windows(2) {
            if window[0] != window[1] {
                return false;
            }
        }
        true
    }
}

impl StoragePressureEngine {
    pub fn new(max_samples: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn record_sample(&mut self, ratio: f64, growth_per_sec: f64) {
        if self.history.len() >= self.max_samples {
            self.history.pop_front();
        }
        self.history.push_back(PressureSample {
            timestamp: chrono::Utc::now(),
            usage_ratio: ratio,
            growth_rate_per_sec: growth_per_sec,
        });
    }

    pub fn analyze(&self) -> PressureReport {
        let current_ratio = self.history.back().map(|s| s.usage_ratio).unwrap_or(0.0);
        let avg_growth = if self.history.len() >= 2 {
            let first = self.history.front().unwrap();
            let last = self.history.back().unwrap();
            let elapsed = (last.timestamp - first.timestamp).num_seconds() as f64;
            if elapsed > 0.0 {
                (last.usage_ratio - first.usage_ratio) / elapsed * 3600.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        let trend = if avg_growth < -0.01 {
            PressureTrend::Declining
        } else if avg_growth > 0.05 {
            PressureTrend::Critical
        } else if avg_growth > 0.01 {
            PressureTrend::Growing
        } else {
            PressureTrend::Stable
        };

        let mut recommendations = Vec::new();
        if current_ratio > 0.85 {
            recommendations.push("Initiate emergency compaction".into());
        }
        if current_ratio > 0.75 {
            recommendations.push("Review retention policies for pruning opportunities".into());
        }
        if matches!(trend, PressureTrend::Critical) {
            recommendations.push("Immediate capacity expansion required".into());
        }

        let exhaustion_hours = if avg_growth > 0.0 {
            let remaining = 1.0 - current_ratio;
            Some(remaining / (avg_growth / 3600.0).max(1e-10))
        } else {
            None
        };

        PressureReport {
            current_ratio,
            trend,
            estimated_exhaustion_hours: exhaustion_hours,
            growth_rate_per_hour: avg_growth,
            recommendations,
        }
    }
}

impl Default for StoragePressureEngine {
    fn default() -> Self {
        Self::new(100)
    }
}

impl SnapshotEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn create_snapshot(
        &self,
        domain: &str,
        stream_id: &str,
        event_count: u64,
        depth: u64,
    ) -> SnapshotManifest {
        let hash = sha2::Sha256::new()
            .chain_update(domain.as_bytes())
            .chain_update(stream_id.as_bytes())
            .chain_update(event_count.to_le_bytes())
            .finalize()
            .to_vec();

        SnapshotManifest {
            snapshot_id: uuid::Uuid::now_v7(),
            domain: domain.to_string(),
            stream_id: stream_id.to_string(),
            event_count,
            snapshot_hash: hash,
            checkpoint_depth: depth,
            taken_at: chrono::Utc::now(),
            deterministic: true,
        }
    }
}

impl CompactionEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn create_plan(target: &str, ctype: CompactionType, savings: u64) -> CompactionPlan {
        CompactionPlan {
            plan_id: uuid::Uuid::now_v7(),
            target: target.to_string(),
            compaction_type: ctype,
            estimated_space_savings: savings,
            replay_safe: matches!(
                ctype,
                CompactionType::CrdtMerge | CompactionType::SnapshotPruning
            ),
        }
    }

    /// Verify compaction safety by checking that the target type is replay-safe
    /// and that space savings are within expected bounds.
    pub fn verify_compaction(&self, plan: &CompactionPlan, actual_savings: u64) -> Vec<String> {
        let mut issues = Vec::new();
        if !plan.replay_safe {
            let msg = format!(
                "Compaction type {:?} on '{}' is not replay-safe",
                plan.compaction_type, plan.target
            );
            warn!("{}", msg);
            issues.push(msg);
        }
        if actual_savings > plan.estimated_space_savings * 2 {
            let msg = format!(
                "Actual savings {} exceeds double estimate {} — possible data loss",
                actual_savings, plan.estimated_space_savings
            );
            warn!("{}", msg);
            issues.push(msg);
        }
        if actual_savings < plan.estimated_space_savings / 10 {
            let msg = format!(
                "Actual savings {} is <10% of estimate {} — compaction ineffective",
                actual_savings, plan.estimated_space_savings
            );
            warn!("{}", msg);
            issues.push(msg);
        }
        issues
    }
}

impl QuotaEnforcer {
    pub fn new() -> Self {
        Self
    }

    pub fn check_quota(domain: &str, used: u64, quota: u64) -> QuotaState {
        let ratio = if quota > 0 {
            used as f64 / quota as f64
        } else {
            0.0
        };
        let action = if ratio >= 1.0 {
            QuotaAction::BlockWrites
        } else if ratio >= 0.95 {
            QuotaAction::EmergencyCompaction
        } else if ratio >= 0.85 {
            QuotaAction::Throttle
        } else if ratio >= 0.75 {
            QuotaAction::Warn
        } else {
            QuotaAction::None
        };

        if ratio > 0.85 {
            info!(domain = %domain, used, quota, ratio, ?action, "Storage quota threshold breached");
        }

        QuotaState {
            domain: domain.to_string(),
            storage_used_bytes: used,
            storage_quota_bytes: quota,
            usage_ratio: ratio,
            over_quota: ratio >= 1.0,
            enforcement_action: action,
        }
    }
}

impl ReplayRetentionEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn create_retention_plan(stream_id: &str, retention_days: u64) -> RetentionPlan {
        let now = chrono::Utc::now();
        RetentionPlan {
            plan_id: uuid::Uuid::now_v7(),
            stream_id: stream_id.to_string(),
            retain_from: now - chrono::Duration::days(retention_days as i64),
            retain_until: now,
            pruning_strategy: PruningStrategy::AgeBased(retention_days),
            archive_target: None,
        }
    }
}

impl Default for StorageGovernor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SnapshotEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CompactionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for QuotaEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ReplayRetentionEngine {
    fn default() -> Self {
        Self::new()
    }
}
