use super::{GovernanceContext, GovernanceViolation, PolicyEvalResult, PolicySeverity};
use std::collections::{HashMap, HashSet};

/// Validates bounded context boundaries and dependency hygiene
pub struct BoundedContextValidator {
    context_boundaries: HashMap<String, BoundaryDefinition>,
    allowed_crossings: HashMap<String, HashSet<String>>,
    forbidden_crossings: HashMap<String, HashSet<String>>,
    shared_kernels: Vec<SharedKernel>,
    validation_rules: Vec<ValidationRule>,
}

#[derive(Debug, Clone)]
pub struct BoundaryDefinition {
    pub context_name: String,
    pub owned_packages: Vec<String>,
    pub exposed_interfaces: Vec<String>,
    pub published_events: Vec<String>,
    pub subscribed_events: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SharedKernel {
    pub name: String,
    pub participating_contexts: Vec<String>,
    pub shared_types: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ValidationRule {
    NoDirectDbAccess(String, Vec<String>),
    MustUseInterface(String, String),
    NoCircularReference(String),
    OwnershipScope(String, Vec<String>),
    EventNamespace(String, String),
}

impl BoundedContextValidator {
    pub fn new() -> Self {
        Self {
            context_boundaries: HashMap::new(),
            allowed_crossings: HashMap::new(),
            forbidden_crossings: HashMap::new(),
            shared_kernels: Vec::new(),
            validation_rules: Vec::new(),
        }
    }

    pub fn register_boundary(&mut self, boundary: BoundaryDefinition) {
        let name = boundary.context_name.clone();
        self.context_boundaries.insert(name, boundary);
    }

    pub fn allow_crossing(&mut self, source: &str, target: &str) {
        self.allowed_crossings
            .entry(source.to_string())
            .or_default()
            .insert(target.to_string());
    }

    pub fn forbid_crossing(&mut self, source: &str, target: &str) {
        self.forbidden_crossings
            .entry(source.to_string())
            .or_default()
            .insert(target.to_string());
    }

    pub fn add_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }

    pub async fn validate_dependency(
        &self,
        source: &str,
        target: &str,
        _context: &GovernanceContext,
    ) -> PolicyEvalResult {
        if let Some(forbidden) = self.forbidden_crossings.get(source) {
            if forbidden.contains(target) {
                return PolicyEvalResult::Violation(GovernanceViolation {
                    policy_id: uuid::Uuid::nil(),
                    policy_name: "NoCrossContextDependency".into(),
                    severity: PolicySeverity::Error,
                    message: format!(
                        "Forbidden cross-context dependency: '{}' depends on '{}'",
                        source, target
                    ),
                    context: {
                        let mut m = HashMap::new();
                        m.insert("source".into(), source.into());
                        m.insert("target".into(), target.into());
                        m
                    },
                    remediations: vec![
                        format!(
                            "Introduce anti-corruption layer between '{}' and '{}'",
                            source, target
                        ),
                        format!(
                            "Refactor '{}' to depend on interface instead of '{}' directly",
                            source, target
                        ),
                    ],
                });
            }
        }

        if let Some(allowed) = self.allowed_crossings.get(source) {
            if !allowed.contains(target) {
                if !self.context_boundaries.contains_key(target) {
                    return PolicyEvalResult::Pass;
                }
                return PolicyEvalResult::Warning(format!(
                    "Undeclared cross-context dependency: '{}' -> '{}'. Dependencies should be explicitly allowed.",
                    source, target
                ));
            }
        }

        PolicyEvalResult::Pass
    }

    pub fn validate_context_isolation(&self, context_name: &str) -> ContextIsolationReport {
        let mut report = ContextIsolationReport {
            context_name: context_name.to_string(),
            boundary_defined: false,
            owned_packages: Vec::new(),
            exposed_interfaces: Vec::new(),
            leak_violations: Vec::new(),
            isolated: false,
        };

        if let Some(boundary) = self.context_boundaries.get(context_name) {
            report.boundary_defined = true;
            report.owned_packages = boundary.owned_packages.clone();
            report.exposed_interfaces = boundary.exposed_interfaces.clone();
        }

        let forbidden = self.forbidden_crossings.get(context_name);
        let has_forbidden = forbidden.map(|f| !f.is_empty()).unwrap_or(false);

        report.isolated = report.boundary_defined && !has_forbidden;

        report
    }

    pub fn validate_shared_kernel(&self, kernel_name: &str) -> Option<&SharedKernel> {
        self.shared_kernels.iter().find(|k| k.name == kernel_name)
    }

    pub fn collect_all_violations(&self) -> Vec<String> {
        let mut violations = Vec::new();

        for (source, targets) in &self.forbidden_crossings {
            for target in targets {
                violations.push(format!("Boundary violation: {} -> {}", source, target));
            }
        }

        for (name, boundary) in &self.context_boundaries {
            for _pkg in &boundary.owned_packages {
                if boundary.exposed_interfaces.is_empty() {
                    violations.push(format!(
                        "Context '{}' owns packages but exposes no interfaces",
                        name
                    ));
                }
            }
        }

        violations
    }
}

#[derive(Debug)]
pub struct ContextIsolationReport {
    pub context_name: String,
    pub boundary_defined: bool,
    pub owned_packages: Vec<String>,
    pub exposed_interfaces: Vec<String>,
    pub leak_violations: Vec<String>,
    pub isolated: bool,
}

impl Default for BoundedContextValidator {
    fn default() -> Self {
        Self::new()
    }
}
