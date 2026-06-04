use super::{GovernanceViolation, PolicyEvalResult, PolicySeverity};
use std::collections::HashMap;
use tracing::info;

/// Governance for federation boundaries, cross-domain routing, and sovereignty policy enforcement
pub struct FederationGovernance {
    domain_registry: HashMap<String, FederatedDomain>,
    boundary_policies: Vec<BoundaryPolicy>,
    cross_domain_routes: Vec<CrossDomainRoute>,
    sovereignty_agreements: Vec<SovereigntyAgreement>,
}

#[derive(Debug, Clone)]
pub struct FederatedDomain {
    pub domain_id: String,
    pub name: String,
    pub domain_type: DomainType,
    pub jurisdiction: String,
    pub allowed_interactions: Vec<String>,
    pub restricted_interactions: Vec<String>,
    pub trust_level: TrustLevel,
    pub schema_version: String,
    pub contact: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainType {
    Sovereign,
    Ministry,
    Regional,
    Institution,
    Edge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    Untrusted,
    Low,
    Medium,
    High,
    Sovereign,
}

#[derive(Debug, Clone)]
pub struct BoundaryPolicy {
    pub policy_id: String,
    pub source_domain: String,
    pub target_domain: String,
    pub allowed_event_types: Vec<String>,
    pub forbidden_event_types: Vec<String>,
    pub require_encryption: bool,
    pub require_signing: bool,
    pub max_throughput: Option<u64>,
    pub require_audit: bool,
}

#[derive(Debug, Clone)]
pub struct CrossDomainRoute {
    pub route_id: String,
    pub source_domain: String,
    pub target_domain: String,
    pub route_type: RouteType,
    pub latency_budget_ms: u64,
    pub sync_frequency_secs: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteType {
    Direct,
    Relayed,
    Federated,
    Sovereign,
}

#[derive(Debug, Clone)]
pub struct SovereigntyAgreement {
    pub agreement_id: String,
    pub domains: Vec<String>,
    pub data_shared: Vec<String>,
    pub data_restricted: Vec<String>,
    pub retention_policy: String,
    pub audit_required: bool,
    pub active: bool,
}

#[derive(Debug)]
pub struct FederationBoundaryReport {
    pub domain: String,
    pub total_boundaries: usize,
    pub violations: Vec<String>,
    pub active_routes: usize,
    pub sovereignty_agreements: usize,
    pub compliant: bool,
}

impl FederationGovernance {
    pub fn new() -> Self {
        Self {
            domain_registry: HashMap::new(),
            boundary_policies: Vec::new(),
            cross_domain_routes: Vec::new(),
            sovereignty_agreements: Vec::new(),
        }
    }

    pub fn register_domain(&mut self, domain: FederatedDomain) {
        let id = domain.domain_id.clone();
        info!(domain = %id, kind = ?domain.domain_type, "Federated domain registered");
        self.domain_registry.insert(id, domain);
    }

    pub fn register_boundary_policy(&mut self, policy: BoundaryPolicy) {
        info!(
            source = %policy.source_domain,
            target = %policy.target_domain,
            "Federation boundary policy registered"
        );
        self.boundary_policies.push(policy);
    }

    pub fn register_route(&mut self, route: CrossDomainRoute) {
        info!(
            source = %route.source_domain,
            target = %route.target_domain,
            kind = ?route.route_type,
            "Cross-domain route registered"
        );
        self.cross_domain_routes.push(route);
    }

    pub fn register_agreement(&mut self, agreement: SovereigntyAgreement) {
        info!(
            domains = ?agreement.domains,
            "Sovereignty agreement registered"
        );
        self.sovereignty_agreements.push(agreement);
    }

    pub fn check_boundary(&self, source: &str, target: &str) -> PolicyEvalResult {
        let source_domain = match self.domain_registry.get(source) {
            Some(d) => d,
            None => {
                return PolicyEvalResult::Violation(GovernanceViolation {
                    policy_id: uuid::Uuid::nil(),
                    policy_name: "FederationBoundary".into(),
                    severity: PolicySeverity::Critical,
                    message: format!(
                        "Source domain '{}' not registered in federation governance",
                        source
                    ),
                    context: {
                        let mut m = HashMap::new();
                        m.insert("source".into(), source.into());
                        m
                    },
                    remediations: vec![format!(
                        "Register domain '{}' in federation governance",
                        source
                    )],
                })
            }
        };

        if source_domain
            .restricted_interactions
            .contains(&target.to_string())
        {
            return PolicyEvalResult::Violation(GovernanceViolation {
                policy_id: uuid::Uuid::nil(),
                policy_name: "FederationBoundary".into(),
                severity: PolicySeverity::Error,
                message: format!(
                    "Domain '{}' is restricted from interacting with '{}'",
                    source, target
                ),
                context: {
                    let mut m = HashMap::new();
                    m.insert("source".into(), source.into());
                    m.insert("target".into(), target.into());
                    m
                },
                remediations: vec![
                    "Review and update restricted interactions list".into(),
                    "Establish sovereignty agreement between domains".into(),
                ],
            });
        }

        PolicyEvalResult::Pass
    }

    pub fn validate_event_routing(
        &self,
        event_type: &str,
        source: &str,
        target: &str,
    ) -> PolicyEvalResult {
        for policy in &self.boundary_policies {
            if policy.source_domain == source && policy.target_domain == target {
                if policy.forbidden_event_types.iter().any(|t| t == event_type) {
                    return PolicyEvalResult::Violation(GovernanceViolation {
                        policy_id: uuid::Uuid::nil(),
                        policy_name: "FederationBoundary".into(),
                        severity: PolicySeverity::Error,
                        message: format!(
                            "Event type '{}' is forbidden from '{}' to '{}'",
                            event_type, source, target
                        ),
                        context: {
                            let mut m = HashMap::new();
                            m.insert("event_type".into(), event_type.into());
                            m.insert("source".into(), source.into());
                            m.insert("target".into(), target.into());
                            m
                        },
                        remediations: vec![
                            "Remove event type from routing policy".into(),
                            "Update boundary policy to allow event type".into(),
                        ],
                    });
                }
                return PolicyEvalResult::Pass;
            }
        }

        PolicyEvalResult::Warning(format!(
            "No boundary policy defined for routing from '{}' to '{}'",
            source, target
        ))
    }

    pub fn analyze_boundaries(&self, domain_id: &str) -> FederationBoundaryReport {
        let mut violations = Vec::new();
        let mut active_routes = 0;

        for route in &self.cross_domain_routes {
            if route.source_domain == domain_id && route.enabled {
                active_routes += 1;
            }
        }

        for policy in &self.boundary_policies {
            if policy.source_domain == domain_id || policy.target_domain == domain_id {
                let target_exists = self.domain_registry.contains_key(&policy.target_domain);
                let source_exists = self.domain_registry.contains_key(&policy.source_domain);
                if !target_exists {
                    violations.push(format!(
                        "Boundary policy references non-existent domain '{}'",
                        policy.target_domain
                    ));
                }
                if !source_exists {
                    violations.push(format!(
                        "Boundary policy references non-existent source domain '{}'",
                        policy.source_domain
                    ));
                }
            }
        }

        let compliant = violations.is_empty();
        FederationBoundaryReport {
            domain: domain_id.to_string(),
            total_boundaries: self.boundary_policies.len(),
            violations,
            active_routes,
            sovereignty_agreements: self.sovereignty_agreements.len(),
            compliant,
        }
    }

    pub fn get_domain(&self, domain_id: &str) -> Option<&FederatedDomain> {
        self.domain_registry.get(domain_id)
    }

    pub fn list_domains(&self) -> Vec<&FederatedDomain> {
        self.domain_registry.values().collect()
    }

    pub fn list_active_routes(&self) -> Vec<&CrossDomainRoute> {
        self.cross_domain_routes
            .iter()
            .filter(|r| r.enabled)
            .collect()
    }
}

impl Default for FederationGovernance {
    fn default() -> Self {
        Self::new()
    }
}
