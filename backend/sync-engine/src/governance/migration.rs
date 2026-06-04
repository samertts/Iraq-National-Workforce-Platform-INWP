use std::collections::HashMap;
use tracing::info;

/// Governance for all database migrations in the sovereign architecture
pub struct MigrationGovernance {
    migrations: HashMap<String, MigrationRecord>,
    migration_policies: Vec<MigrationPolicy>,
    execution_history: Vec<MigrationExecution>,
}

#[derive(Debug, Clone)]
pub struct MigrationRecord {
    pub migration_id: String,
    pub version: semver::Version,
    pub description: String,
    pub migration_type: MigrationType,
    pub checksum: Vec<u8>,
    pub author: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub dependencies: Vec<String>,
    pub rollback_script: Option<String>,
    pub replay_safe: bool,
    pub reviewed: bool,
    pub approved_by: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationType {
    Schema,
    Data,
    Index,
    Partition,
    Replication,
    Configuration,
    Sovereignty,
}

#[derive(Debug, Clone)]
pub struct MigrationPolicy {
    pub policy_id: String,
    pub require_review: bool,
    pub require_replay_safe: bool,
    pub require_rollback: bool,
    pub max_per_deployment: usize,
    pub require_approval: bool,
    pub require_checksum_verification: bool,
}

#[derive(Debug, Clone)]
pub struct MigrationExecution {
    pub execution_id: uuid::Uuid,
    pub migration_id: String,
    pub status: ExecutionStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
    pub duration_ms: Option<u64>,
    pub executed_by: String,
    pub environment: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug)]
pub struct MigrationSafetyReport {
    pub migration_id: String,
    pub safe: bool,
    pub issues: Vec<MigrationIssue>,
    pub replay_compatible: bool,
    pub rollback_available: bool,
}

#[derive(Debug)]
pub struct MigrationIssue {
    pub severity: IssueSeverity,
    pub message: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Info,
    Warning,
    Blocker,
}

impl MigrationGovernance {
    pub fn new() -> Self {
        Self {
            migrations: HashMap::new(),
            migration_policies: Vec::new(),
            execution_history: Vec::new(),
        }
    }

    pub fn register_migration(&mut self, migration: MigrationRecord) {
        let id = migration.migration_id.clone();
        info!(
            migration = %id,
            version = %migration.version,
            kind = ?migration.migration_type,
            "Migration registered"
        );
        self.migrations.insert(id, migration);
    }

    pub fn register_policy(&mut self, policy: MigrationPolicy) {
        info!(policy = %policy.policy_id, "Migration policy registered");
        self.migration_policies.push(policy);
    }

    pub fn validate_migration(&self, migration_id: &str) -> MigrationSafetyReport {
        let mut issues = Vec::new();
        let mut replay_compatible = true;
        let mut rollback_available = false;

        if let Some(migration) = self.migrations.get(migration_id) {
            for policy in &self.migration_policies {
                if policy.require_rollback && migration.rollback_script.is_none() {
                    issues.push(MigrationIssue {
                        severity: IssueSeverity::Blocker,
                        message: format!(
                            "Migration '{}' requires a rollback script but none provided",
                            migration_id
                        ),
                        remediation: Some("Provide a rollback script for this migration".into()),
                    });
                } else if migration.rollback_script.is_some() {
                    rollback_available = true;
                }

                if policy.require_replay_safe && !migration.replay_safe {
                    issues.push(MigrationIssue {
                        severity: IssueSeverity::Warning,
                        message: format!("Migration '{}' is not marked replay-safe", migration_id),
                        remediation: Some(
                            "Mark migration as replay-safe or provide replay compatibility proof"
                                .into(),
                        ),
                    });
                    replay_compatible = false;
                }

                if policy.require_review && !migration.reviewed {
                    issues.push(MigrationIssue {
                        severity: IssueSeverity::Blocker,
                        message: format!(
                            "Migration '{}' requires review but has not been reviewed",
                            migration_id
                        ),
                        remediation: Some("Schedule migration review before deployment".into()),
                    });
                }

                if policy.require_approval && migration.approved_by.is_none() {
                    issues.push(MigrationIssue {
                        severity: IssueSeverity::Blocker,
                        message: format!(
                            "Migration '{}' requires approval but has not been approved",
                            migration_id
                        ),
                        remediation: Some("Obtain approval for this migration".into()),
                    });
                }
            }

            let mut checksum_valid = true;
            if let Some(policy) = self.migration_policies.first() {
                if policy.require_checksum_verification && migration.checksum.is_empty() {
                    issues.push(MigrationIssue {
                        severity: IssueSeverity::Warning,
                        message: format!(
                            "Migration '{}' has no checksum for integrity verification",
                            migration_id
                        ),
                        remediation: Some("Generate and attach a SHA-256 checksum".into()),
                    });
                    checksum_valid = false;
                }
            }

            if !checksum_valid {
                replay_compatible = false;
            }
        } else {
            issues.push(MigrationIssue {
                severity: IssueSeverity::Blocker,
                message: format!(
                    "Migration '{}' is not registered in the migration registry",
                    migration_id
                ),
                remediation: Some("Register the migration before attempting deployment".into()),
            });
        }

        MigrationSafetyReport {
            migration_id: migration_id.to_string(),
            safe: issues
                .iter()
                .all(|i| matches!(i.severity, IssueSeverity::Info | IssueSeverity::Warning)),
            issues,
            replay_compatible,
            rollback_available,
        }
    }

    pub fn record_execution(&mut self, execution: MigrationExecution) {
        info!(
            migration = %execution.migration_id,
            status = ?execution.status,
            "Migration execution recorded"
        );
        self.execution_history.push(execution);
    }

    pub fn get_migration(&self, migration_id: &str) -> Option<&MigrationRecord> {
        self.migrations.get(migration_id)
    }

    pub fn list_pending_migrations(&self) -> Vec<&MigrationRecord> {
        let executed: Vec<&str> = self
            .execution_history
            .iter()
            .filter(|e| matches!(e.status, ExecutionStatus::Completed))
            .map(|e| e.migration_id.as_str())
            .collect();

        self.migrations
            .values()
            .filter(|m| !executed.contains(&m.migration_id.as_str()))
            .collect()
    }

    pub fn get_execution_history(&self) -> &[MigrationExecution] {
        &self.execution_history
    }
}

impl Default for MigrationGovernance {
    fn default() -> Self {
        Self::new()
    }
}
