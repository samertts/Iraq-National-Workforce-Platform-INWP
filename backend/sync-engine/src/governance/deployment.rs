use super::{GovernanceViolation, PolicyEvalResult, PolicySeverity};
use std::collections::HashMap;
use tracing::info;

/// Governance for deployment policies, infrastructure validation, and topology enforcement
pub struct DeploymentGovernance {
    component_registry: HashMap<String, DeploymentComponent>,
    environment_registry: HashMap<String, DeploymentEnvironment>,
    deployment_policies: Vec<DeploymentPolicy>,
    release_history: Vec<ReleaseRecord>,
    tag_policies: Vec<TagPolicy>,
}

#[derive(Debug, Clone)]
pub struct DeploymentComponent {
    pub component_id: String,
    pub name: String,
    pub component_type: ComponentType,
    pub owner: String,
    pub allowed_environments: Vec<String>,
    pub required_tags: Vec<String>,
    pub dependencies: Vec<String>,
    pub security_scan_required: bool,
    pub sbom_required: bool,
    pub signing_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    SyncEngine,
    IdentityService,
    FederationService,
    PolicyEngine,
    AuthService,
    TrustService,
    CertificateService,
    EdgeRuntime,
    ControlPlane,
    Governance,
}

#[derive(Debug, Clone)]
pub struct DeploymentEnvironment {
    pub environment_id: String,
    pub name: String,
    pub tier: EnvironmentTier,
    pub region: String,
    pub allowed_components: Vec<String>,
    pub requires_approval: bool,
    pub requires_audit: bool,
    pub max_concurrent_deployments: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentTier {
    Development,
    Staging,
    Regional,
    National,
    Sovereign,
}

#[derive(Debug, Clone)]
pub struct DeploymentPolicy {
    pub policy_id: String,
    pub name: String,
    pub require_signature_verification: bool,
    pub require_sbom: bool,
    pub require_security_scan: bool,
    pub require_governance_approval: bool,
    pub require_replay_safety_check: bool,
    pub require_schema_validation: bool,
    pub max_rollout_percentage: u8,
    pub cooldown_minutes: u64,
}

#[derive(Debug, Clone)]
pub struct ReleaseRecord {
    pub release_id: uuid::Uuid,
    pub component_id: String,
    pub version: String,
    pub environment: String,
    pub deployed_at: chrono::DateTime<chrono::Utc>,
    pub deployed_by: String,
    pub status: ReleaseStatus,
    pub artifact_hash: Vec<u8>,
    pub approval: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseStatus {
    Pending,
    Approved,
    Deploying,
    Deployed,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone)]
pub struct TagPolicy {
    pub tag_name: String,
    pub required_components: Vec<String>,
    pub forbidden_components: Vec<String>,
    pub min_version: Option<String>,
}

#[derive(Debug)]
pub struct DeploymentValidationReport {
    pub component: String,
    pub environment: String,
    pub valid: bool,
    pub policy_checks: Vec<PolicyCheckResult>,
    pub violations: Vec<String>,
}

#[derive(Debug)]
pub struct PolicyCheckResult {
    pub check_name: String,
    pub passed: bool,
    pub message: String,
}

impl DeploymentGovernance {
    pub fn new() -> Self {
        Self {
            component_registry: HashMap::new(),
            environment_registry: HashMap::new(),
            deployment_policies: Vec::new(),
            release_history: Vec::new(),
            tag_policies: Vec::new(),
        }
    }

    pub fn register_component(&mut self, component: DeploymentComponent) {
        let id = component.component_id.clone();
        info!(component = %id, kind = ?component.component_type, "Deployment component registered");
        self.component_registry.insert(id, component);
    }

    pub fn register_environment(&mut self, env: DeploymentEnvironment) {
        let id = env.environment_id.clone();
        info!(environment = %id, tier = ?env.tier, "Deployment environment registered");
        self.environment_registry.insert(id, env);
    }

    pub fn register_policy(&mut self, policy: DeploymentPolicy) {
        info!(policy = %policy.policy_id, "Deployment policy registered");
        self.deployment_policies.push(policy);
    }

    pub fn record_release(&mut self, release: ReleaseRecord) {
        info!(
            component = %release.component_id,
            version = %release.version,
            environment = %release.environment,
            "Release recorded"
        );
        self.release_history.push(release);
    }

    pub fn validate_tag(&self, component: &str, tag: &str) -> PolicyEvalResult {
        for tp in &self.tag_policies {
            if tp.tag_name == tag {
                if tp.forbidden_components.iter().any(|c| c == component) {
                    return PolicyEvalResult::Violation(GovernanceViolation {
                        policy_id: uuid::Uuid::nil(),
                        policy_name: "DeploymentTag".into(),
                        severity: PolicySeverity::Error,
                        message: format!(
                            "Component '{}' is forbidden from using tag '{}'",
                            component, tag
                        ),
                        context: {
                            let mut m = HashMap::new();
                            m.insert("component".into(), component.into());
                            m.insert("tag".into(), tag.into());
                            m
                        },
                        remediations: vec![
                            format!("Remove tag '{}' from component '{}'", tag, component),
                            "Update tag policy to allow this component".into(),
                        ],
                    });
                }
                return PolicyEvalResult::Pass;
            }
        }
        PolicyEvalResult::Pass
    }

    pub fn validate_deployment(
        &self,
        component_id: &str,
        environment_id: &str,
        _version: &str,
    ) -> DeploymentValidationReport {
        let mut violations = Vec::new();
        let mut policy_checks = Vec::new();

        let component = self.component_registry.get(component_id);
        let environment = self.environment_registry.get(environment_id);

        if let Some(env) = environment {
            if let Some(comp) = component {
                let allowed = env.allowed_components.is_empty()
                    || env.allowed_components.contains(&comp.name);
                policy_checks.push(PolicyCheckResult {
                    check_name: "Environment allowed components".into(),
                    passed: allowed,
                    message: if allowed {
                        format!(
                            "Component '{}' allowed in environment '{}'",
                            comp.name, env.name
                        )
                    } else {
                        format!(
                            "Component '{}' not allowed in environment '{}'",
                            comp.name, env.name
                        )
                    },
                });
                if !allowed {
                    violations.push(format!(
                        "Component '{}' not in allowed list for environment '{}'",
                        comp.name, env.name
                    ));
                }
            }
        }

        for policy in &self.deployment_policies {
            if policy.require_sbom {
                let has_sbom = component.map(|c| c.sbom_required).unwrap_or(false);
                policy_checks.push(PolicyCheckResult {
                    check_name: "SBOM requirement".into(),
                    passed: has_sbom,
                    message: if has_sbom {
                        "SBOM present".into()
                    } else {
                        "SBOM missing".into()
                    },
                });
                if !has_sbom {
                    violations.push("SBOM is required but not present".into());
                }
            }

            if policy.require_security_scan {
                let scanned = component.map(|c| c.security_scan_required).unwrap_or(false);
                policy_checks.push(PolicyCheckResult {
                    check_name: "Security scan".into(),
                    passed: scanned,
                    message: if scanned {
                        "Security scan passed".into()
                    } else {
                        "Security scan required".into()
                    },
                });
                if !scanned {
                    violations.push("Security scan is required but was not performed".into());
                }
            }

            if policy.require_signature_verification {
                let signed = component.map(|c| c.signing_required).unwrap_or(false);
                policy_checks.push(PolicyCheckResult {
                    check_name: "Signature verification".into(),
                    passed: signed,
                    message: if signed {
                        "Artifact signed".into()
                    } else {
                        "Artifact not signed".into()
                    },
                });
                if !signed {
                    violations.push("Artifact signing is required".into());
                }
            }
        }

        DeploymentValidationReport {
            component: component_id.to_string(),
            environment: environment_id.to_string(),
            valid: violations.is_empty(),
            policy_checks,
            violations,
        }
    }

    pub fn get_component(&self, component_id: &str) -> Option<&DeploymentComponent> {
        self.component_registry.get(component_id)
    }

    pub fn get_environment(&self, environment_id: &str) -> Option<&DeploymentEnvironment> {
        self.environment_registry.get(environment_id)
    }

    pub fn get_release_history(&self) -> &[ReleaseRecord] {
        &self.release_history
    }

    pub fn get_deployments_in_environment(&self, environment_id: &str) -> Vec<&ReleaseRecord> {
        self.release_history
            .iter()
            .filter(|r| r.environment == environment_id)
            .collect()
    }
}

impl Default for DeploymentGovernance {
    fn default() -> Self {
        Self::new()
    }
}
