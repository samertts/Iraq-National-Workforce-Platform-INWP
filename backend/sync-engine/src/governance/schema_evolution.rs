use crate::error::SyncResult;
use std::collections::HashMap;
use tracing::info;

/// Schema evolution engine — manages safe, replay-compatible schema migrations across the sovereign federation
pub struct SchemaEvolutionEngine {
    active_evolutions: Vec<SchemaEvolution>,
    evolution_history: Vec<EvolutionExecution>,
    negotiation_cache: HashMap<String, FederationNegotiation>,
}

#[derive(Debug, Clone)]
pub struct SchemaEvolution {
    pub evolution_id: uuid::Uuid,
    pub contract_id: String,
    pub from_version: semver::Version,
    pub to_version: semver::Version,
    pub evolution_type: EvolutionType,
    pub migration_strategy: MigrationStrategy,
    pub requires_federation_negotiation: bool,
    pub replay_safe: bool,
    pub approved: bool,
    pub approved_by: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionType {
    Additive,
    Deprecative,
    Renaming,
    Restructuring,
    Split,
    Merge,
    SovereignPolicyChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationStrategy {
    LazyMigration,
    EagerMigration,
    DualWrite,
    FederationNegotiated,
    CoordinatedRollout,
}

#[derive(Debug, Clone)]
pub struct EvolutionExecution {
    pub execution_id: uuid::Uuid,
    pub evolution_id: uuid::Uuid,
    pub status: EvolutionStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub domains_updated: Vec<String>,
    pub events_migrated: u64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvolutionStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone)]
pub struct FederationNegotiation {
    pub contract_id: String,
    pub proposing_domain: String,
    pub responding_domain: String,
    pub proposed_version: semver::Version,
    pub current_version: semver::Version,
    pub accepted: bool,
    pub negotiated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub conditions: Vec<String>,
}

#[derive(Debug)]
pub struct EvolutionSafetyReport {
    pub evolution_id: uuid::Uuid,
    pub safe: bool,
    pub replay_safe: bool,
    pub backward_compatible: bool,
    pub corruption_risk: f64,
    pub federation_impact: Vec<String>,
    pub rollback_possible: bool,
    pub issues: Vec<String>,
}

impl SchemaEvolutionEngine {
    pub fn new() -> Self {
        Self {
            active_evolutions: Vec::new(),
            evolution_history: Vec::new(),
            negotiation_cache: HashMap::new(),
        }
    }

    pub fn propose_evolution(&mut self, evolution: SchemaEvolution) -> EvolutionSafetyReport {
        let mut issues = Vec::new();
        let mut federation_impact = Vec::new();

        if evolution.from_version >= evolution.to_version {
            issues.push(format!(
                "Version must increase: {} -> {}",
                evolution.from_version, evolution.to_version
            ));
        }

        if evolution.requires_federation_negotiation && !evolution.approved {
            federation_impact
                .push("Evolution requires federation negotiation before deployment".into());
        }

        if !evolution.replay_safe {
            issues.push(
                "Evolution is not replay-safe — will produce divergent state on replay".into(),
            );
        }

        let report = EvolutionSafetyReport {
            evolution_id: evolution.evolution_id,
            safe: issues.is_empty(),
            replay_safe: evolution.replay_safe,
            backward_compatible: matches!(evolution.evolution_type, EvolutionType::Additive),
            corruption_risk: if evolution.replay_safe { 0.0 } else { 0.85 },
            federation_impact,
            rollback_possible: evolution.from_version > semver::Version::new(0, 0, 0),
            issues: issues.clone(),
        };

        if report.safe {
            info!(
                evolution = %evolution.evolution_id,
                contract = %evolution.contract_id,
                from = %evolution.from_version,
                to = %evolution.to_version,
                "Schema evolution proposed and validated"
            );
            self.active_evolutions.push(evolution);
        }

        report
    }

    pub fn execute_evolution(
        &mut self,
        evolution_id: uuid::Uuid,
    ) -> SyncResult<EvolutionExecution> {
        let evolution = self
            .active_evolutions
            .iter()
            .find(|e| e.evolution_id == evolution_id)
            .ok_or_else(|| {
                crate::error::SyncEngineError::Internal(format!(
                    "Evolution '{}' not found in active evolutions",
                    evolution_id
                ))
            })?
            .clone();

        if !evolution.approved {
            return Err(crate::error::SyncEngineError::Internal(format!(
                "Evolution '{}' has not been approved",
                evolution_id
            )));
        }

        let execution = EvolutionExecution {
            execution_id: uuid::Uuid::now_v7(),
            evolution_id,
            status: EvolutionStatus::InProgress,
            started_at: chrono::Utc::now(),
            completed_at: None,
            domains_updated: Vec::new(),
            events_migrated: 0,
            errors: Vec::new(),
        };

        info!(
            evolution = %evolution_id,
            "Schema evolution execution started"
        );

        Ok(execution)
    }

    pub fn complete_evolution(
        &mut self,
        execution_id: uuid::Uuid,
        domains_updated: Vec<String>,
        events_migrated: u64,
    ) {
        let execution = EvolutionExecution {
            execution_id,
            evolution_id: uuid::Uuid::nil(),
            status: EvolutionStatus::Completed,
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
            domains_updated,
            events_migrated,
            errors: Vec::new(),
        };
        self.evolution_history.push(execution);
    }

    pub fn negotiate_federation(
        &mut self,
        contract_id: &str,
        proposing_domain: &str,
        responding_domain: &str,
        proposed: &semver::Version,
        current: &semver::Version,
    ) -> FederationNegotiation {
        let key = format!(
            "{}:{}->{}",
            contract_id, proposing_domain, responding_domain
        );
        let negotiation = FederationNegotiation {
            contract_id: contract_id.to_string(),
            proposing_domain: proposing_domain.to_string(),
            responding_domain: responding_domain.to_string(),
            proposed_version: proposed.clone(),
            current_version: current.clone(),
            accepted: proposed >= current,
            negotiated_at: Some(chrono::Utc::now()),
            conditions: if proposed >= current {
                vec![]
            } else {
                vec![format!(
                    "Proposed version {} is lower than current {}",
                    proposed, current
                )]
            },
        };
        self.negotiation_cache.insert(key, negotiation.clone());
        negotiation
    }

    pub fn get_active_evolutions(&self) -> &[SchemaEvolution] {
        &self.active_evolutions
    }

    pub fn get_history(&self) -> &[EvolutionExecution] {
        &self.evolution_history
    }

    pub fn validate_temporal_schema(
        &self,
        contract_id: &str,
        at_version: &semver::Version,
    ) -> bool {
        self.active_evolutions
            .iter()
            .any(|e| e.contract_id == contract_id && e.to_version == *at_version && e.approved)
    }
}

impl Default for SchemaEvolutionEngine {
    fn default() -> Self {
        Self::new()
    }
}
