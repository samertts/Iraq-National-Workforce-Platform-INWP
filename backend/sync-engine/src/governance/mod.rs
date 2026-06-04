pub mod analyzer;
pub mod compatibility;
pub mod deployment;
pub mod event_registry;
pub mod federation;
pub mod migration;
pub mod registry;
pub mod replay;
pub mod schema;
pub mod schema_evolution;
pub mod sovereignty;
pub mod sovereignty_ext;
pub mod validator;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Central orchestrator for all sovereign governance subsystems.
/// Every governance decision is logged, auditable, and enforceable.
pub struct GovernanceEngine {
    registry: Arc<RwLock<registry::ArchitectureRegistry>>,
    analyzer: Arc<RwLock<analyzer::DependencyAnalyzer>>,
    validator: Arc<RwLock<validator::BoundedContextValidator>>,
    schema_gov: Arc<RwLock<schema::SchemaGovernance>>,
    migration_gov: Arc<RwLock<migration::MigrationGovernance>>,
    replay_gov: Arc<RwLock<replay::ReplayGovernance>>,
    federation_gov: Arc<RwLock<federation::FederationGovernance>>,
    deployment_gov: Arc<RwLock<deployment::DeploymentGovernance>>,
    sovereignty_reg: Arc<RwLock<sovereignty::SovereigntyRegistry>>,
    audit_log: Arc<RwLock<Vec<GovernanceAuditEntry>>>,
    policy_rules: Arc<RwLock<Vec<GovernancePolicy>>>,
    enforcement_mode: Arc<RwLock<EnforcementMode>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementMode {
    LogOnly,
    WarnOnly,
    Enforce,
    StrictEnforce,
}

#[derive(Debug, Clone)]
pub struct GovernancePolicy {
    pub policy_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub severity: PolicySeverity,
    pub rule: PolicyRule,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicySeverity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone)]
pub enum PolicyRule {
    NoCrossContextDependency(String, String),
    RequiredInterface(String, String),
    ForbiddenDependency(String, String),
    MaxCouplingScore(f64),
    MandatoryOwner(String),
    SchemaEvolutionRule(String, schema::EvolutionRule),
    MigrationSafetyRule(String),
    ReplayDeterminism(String),
    FederationBoundary(String, String),
    DeploymentTag(String, String),
    SovereigntyResidency(String, Vec<String>),
}

#[derive(Debug, Clone)]
pub struct GovernanceAuditEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub actor: String,
    pub action: String,
    pub target: String,
    pub decision: GovernanceDecision,
    pub details: String,
}

#[derive(Debug, Clone)]
pub enum GovernanceDecision {
    Allowed,
    Denied(String),
    Warning(String),
    Logged,
}

#[derive(Debug, Clone)]
pub struct GovernanceReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub policies_evaluated: u64,
    pub violations: Vec<GovernanceViolation>,
    pub warnings: Vec<String>,
    pub passed: bool,
}

#[derive(Debug, Clone)]
pub struct GovernanceViolation {
    pub policy_id: uuid::Uuid,
    pub policy_name: String,
    pub severity: PolicySeverity,
    pub message: String,
    pub context: HashMap<String, String>,
    pub remediations: Vec<String>,
}

impl GovernanceEngine {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(registry::ArchitectureRegistry::new())),
            analyzer: Arc::new(RwLock::new(analyzer::DependencyAnalyzer::new())),
            validator: Arc::new(RwLock::new(validator::BoundedContextValidator::new())),
            schema_gov: Arc::new(RwLock::new(schema::SchemaGovernance::new())),
            migration_gov: Arc::new(RwLock::new(migration::MigrationGovernance::new())),
            replay_gov: Arc::new(RwLock::new(replay::ReplayGovernance::new())),
            federation_gov: Arc::new(RwLock::new(federation::FederationGovernance::new())),
            deployment_gov: Arc::new(RwLock::new(deployment::DeploymentGovernance::new())),
            sovereignty_reg: Arc::new(RwLock::new(sovereignty::SovereigntyRegistry::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            policy_rules: Arc::new(RwLock::new(Vec::new())),
            enforcement_mode: Arc::new(RwLock::new(EnforcementMode::Enforce)),
        }
    }

    pub async fn set_enforcement_mode(&self, mode: EnforcementMode) {
        let mut current = self.enforcement_mode.write().await;
        *current = mode;
        info!(?mode, "Governance enforcement mode changed");
    }

    pub async fn add_policy(&self, policy: GovernancePolicy) {
        let mut policies = self.policy_rules.write().await;
        policies.push(policy);
    }

    pub async fn evaluate(&self, context: &GovernanceContext) -> GovernanceReport {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();
        let policies = self.policy_rules.read().await;
        let mode = *self.enforcement_mode.read().await;

        for policy in policies.iter().filter(|p| p.enabled) {
            let result = self.evaluate_policy(policy, context).await;
            match result {
                PolicyEvalResult::Pass => {}
                PolicyEvalResult::Violation(ref v) => {
                    violations.push(v.clone());
                    let entry = GovernanceAuditEntry {
                        timestamp: chrono::Utc::now(),
                        actor: context.actor.clone(),
                        action: context.action.clone(),
                        target: context.target.clone(),
                        decision: GovernanceDecision::Denied(v.message.clone()),
                        details: format!("Policy '{}' violated: {}", policy.name, v.message),
                    };
                    self.audit_log.write().await.push(entry);

                    if matches!(mode, EnforcementMode::StrictEnforce) {
                        panic!("Governance violation in strict enforce mode: {}", v.message);
                    }
                }
                PolicyEvalResult::Warning(ref w) => {
                    let msg = w.clone();
                    warnings.push(msg.clone());
                    let entry = GovernanceAuditEntry {
                        timestamp: chrono::Utc::now(),
                        actor: context.actor.clone(),
                        action: context.action.clone(),
                        target: context.target.clone(),
                        decision: GovernanceDecision::Warning(msg),
                        details: format!("Policy '{}' warning: {}", policy.name, w),
                    };
                    self.audit_log.write().await.push(entry);
                }
            }
        }

        let passed = violations.is_empty() && !matches!(mode, EnforcementMode::StrictEnforce);
        GovernanceReport {
            timestamp: chrono::Utc::now(),
            policies_evaluated: policies.iter().filter(|p| p.enabled).count() as u64,
            violations,
            warnings,
            passed,
        }
    }

    async fn evaluate_policy(
        &self,
        policy: &GovernancePolicy,
        context: &GovernanceContext,
    ) -> PolicyEvalResult {
        match &policy.rule {
            PolicyRule::NoCrossContextDependency(source, target) => {
                self.validator
                    .read()
                    .await
                    .validate_dependency(source, target, context)
                    .await
            }
            PolicyRule::RequiredInterface(context_name, interface) => self
                .registry
                .read()
                .await
                .verify_interface(context_name, interface),
            PolicyRule::ForbiddenDependency(source, target) => {
                if self.analyzer.read().await.has_dependency(source, target) {
                    PolicyEvalResult::Violation(GovernanceViolation {
                        policy_id: policy.policy_id,
                        policy_name: policy.name.clone(),
                        severity: policy.severity,
                        message: format!("Forbidden dependency from '{}' to '{}'", source, target),
                        context: HashMap::new(),
                        remediations: vec![format!(
                            "Remove dependency from {} to {}",
                            source, target
                        )],
                    })
                } else {
                    PolicyEvalResult::Pass
                }
            }
            PolicyRule::MaxCouplingScore(max) => {
                let score = self
                    .analyzer
                    .read()
                    .await
                    .compute_coupling_score(context.target.as_str());
                if score > *max {
                    PolicyEvalResult::Violation(GovernanceViolation {
                        policy_id: policy.policy_id,
                        policy_name: policy.name.clone(),
                        severity: policy.severity,
                        message: format!("Coupling score {:.2} exceeds max {:.2}", score, max),
                        context: HashMap::new(),
                        remediations: vec!["Reduce coupling through interface abstraction".into()],
                    })
                } else {
                    PolicyEvalResult::Pass
                }
            }
            PolicyRule::MandatoryOwner(context_name) => {
                if self.registry.read().await.get_owner(context_name).is_none() {
                    PolicyEvalResult::Violation(GovernanceViolation {
                        policy_id: policy.policy_id,
                        policy_name: policy.name.clone(),
                        severity: policy.severity,
                        message: format!("Context '{}' has no registered owner", context_name),
                        context: HashMap::new(),
                        remediations: vec![format!(
                            "Register an owner for context '{}'",
                            context_name
                        )],
                    })
                } else {
                    PolicyEvalResult::Pass
                }
            }
            PolicyRule::SchemaEvolutionRule(contract_id, rule) => self
                .schema_gov
                .read()
                .await
                .check_evolution_rule(contract_id, rule),
            PolicyRule::MigrationSafetyRule(_) => PolicyEvalResult::Pass,
            PolicyRule::ReplayDeterminism(stream) => {
                self.replay_gov.read().await.check_determinism(stream)
            }
            PolicyRule::FederationBoundary(domain, allowed) => self
                .federation_gov
                .read()
                .await
                .check_boundary(domain, allowed),
            PolicyRule::DeploymentTag(component, tag) => self
                .deployment_gov
                .read()
                .await
                .validate_tag(component, tag),
            PolicyRule::SovereigntyResidency(zone, allowed_regions) => self
                .sovereignty_reg
                .read()
                .await
                .check_residency(zone, allowed_regions),
        }
    }

    pub async fn get_audit_log(&self) -> Vec<GovernanceAuditEntry> {
        self.audit_log.read().await.clone()
    }

    pub async fn registry(&self) -> &Arc<RwLock<registry::ArchitectureRegistry>> {
        &self.registry
    }

    pub async fn analyzer(&self) -> &Arc<RwLock<analyzer::DependencyAnalyzer>> {
        &self.analyzer
    }

    pub async fn schema_governance(&self) -> &Arc<RwLock<schema::SchemaGovernance>> {
        &self.schema_gov
    }

    pub async fn federation_governance(&self) -> &Arc<RwLock<federation::FederationGovernance>> {
        &self.federation_gov
    }

    pub async fn sovereignty_registry(&self) -> &Arc<RwLock<sovereignty::SovereigntyRegistry>> {
        &self.sovereignty_reg
    }
}

#[derive(Debug, Clone)]
pub struct GovernanceContext {
    pub actor: String,
    pub action: String,
    pub target: String,
    pub parameters: HashMap<String, String>,
}

pub enum PolicyEvalResult {
    Pass,
    Violation(GovernanceViolation),
    Warning(String),
}

impl Default for GovernanceEngine {
    fn default() -> Self {
        Self::new()
    }
}
