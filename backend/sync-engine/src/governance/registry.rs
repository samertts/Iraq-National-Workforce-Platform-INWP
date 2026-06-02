use std::collections::HashMap;
use tracing::info;

/// Registered bounded context within the sovereign architecture
#[derive(Debug, Clone)]
pub struct BoundedContext {
    pub name: String,
    pub owner: String,
    pub domain: String,
    pub context_type: ContextType,
    pub interfaces: Vec<ContextInterface>,
    pub allowed_dependencies: Vec<String>,
    pub forbidden_dependencies: Vec<String>,
    pub events_published: Vec<String>,
    pub events_subscribed: Vec<String>,
    pub sovereignty_zone: String,
    pub status: ContextStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextType {
    Core,
    Supporting,
    Generic,
    Sovereignty,
    Federation,
}

#[derive(Debug, Clone)]
pub struct ContextInterface {
    pub name: String,
    pub version: String,
    pub protocol: String,
    pub direction: InterfaceDirection,
    pub schema_contract: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceDirection {
    Inbound,
    Outbound,
    Bidirectional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextStatus {
    Active,
    Deprecated,
    Retired,
    UnderReview,
}

/// Central registry for all bounded contexts and their relationships
pub struct ArchitectureRegistry {
    contexts: HashMap<String, BoundedContext>,
    context_maps: HashMap<String, Vec<ContextMapEntry>>,
    ownership_registry: HashMap<String, String>,
    anti_corruption_layers: Vec<AntiCorruptionLayer>,
}

#[derive(Debug, Clone)]
pub struct ContextMapEntry {
    pub source: String,
    pub target: String,
    pub relationship: ContextRelationship,
    pub acl_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextRelationship {
    UpstreamDownstream,
    Partnership,
    SharedKernel,
    SeparateWays,
    OpenHostService,
    PublishedLanguage,
    Conformist,
    AntiCorruptionLayer,
}

#[derive(Debug, Clone)]
pub struct AntiCorruptionLayer {
    pub name: String,
    pub source_context: String,
    pub target_context: String,
    pub translation_rules: Vec<String>,
    pub active: bool,
}

impl ArchitectureRegistry {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            context_maps: HashMap::new(),
            ownership_registry: HashMap::new(),
            anti_corruption_layers: Vec::new(),
        }
    }

    pub fn register_context(&mut self, context: BoundedContext) {
        let name = context.name.clone();
        self.ownership_registry.insert(name.clone(), context.owner.clone());
        self.contexts.insert(name.clone(), context);
        info!(context = %name, "Bounded context registered in architecture registry");
    }

    pub fn register_context_map(&mut self, entry: ContextMapEntry) {
        let source = entry.source.clone();
        self.context_maps.entry(source).or_default().push(entry);
    }

    pub fn register_acl(&mut self, acl: AntiCorruptionLayer) {
        info!(
            source = %acl.source_context,
            target = %acl.target_context,
            "Anti-corruption layer registered"
        );
        self.anti_corruption_layers.push(acl);
    }

    pub fn get_context(&self, name: &str) -> Option<&BoundedContext> {
        self.contexts.get(name)
    }

    pub fn get_owner(&self, context_name: &str) -> Option<&String> {
        self.ownership_registry.get(context_name)
    }

    pub fn verify_interface(&self, context_name: &str, interface: &str) -> super::PolicyEvalResult {
        match self.contexts.get(context_name) {
            Some(ctx) => {
                if ctx.interfaces.iter().any(|i| i.name == interface) {
                    super::PolicyEvalResult::Pass
                } else {
                    super::PolicyEvalResult::Violation(super::GovernanceViolation {
                        policy_id: uuid::Uuid::nil(),
                        policy_name: "RequiredInterface".into(),
                        severity: super::PolicySeverity::Error,
                        message: format!("Context '{}' missing required interface '{}'", context_name, interface),
                        context: std::collections::HashMap::new(),
                        remediations: vec![format!("Implement interface '{}' on context '{}'", interface, context_name)],
                    })
                }
            }
            None => super::PolicyEvalResult::Violation(super::GovernanceViolation {
                policy_id: uuid::Uuid::nil(),
                policy_name: "RequiredInterface".into(),
                severity: super::PolicySeverity::Critical,
                message: format!("Context '{}' not found in architecture registry", context_name),
                context: std::collections::HashMap::new(),
                remediations: vec![format!("Register context '{}' in architecture registry", context_name)],
            }),
        }
    }

    pub fn list_contexts(&self) -> Vec<&BoundedContext> {
        self.contexts.values().collect()
    }

    pub fn list_acls(&self) -> &[AntiCorruptionLayer] {
        &self.anti_corruption_layers
    }

    pub fn analyze_context_boundaries(&self, name: &str) -> BoundaryAnalysis {
        let mut analysis = BoundaryAnalysis {
            context_name: name.to_string(),
            incoming_dependencies: Vec::new(),
            outgoing_dependencies: Vec::new(),
            violations: Vec::new(),
            missing_acls: Vec::new(),
        };

        if let Some(ctx) = self.contexts.get(name) {
            for dep in &ctx.allowed_dependencies {
                if !self.contexts.contains_key(dep) {
                    analysis.violations.push(format!("Allowed dependency '{}' is not a registered context", dep));
                }
            }
            if let Some(maps) = self.context_maps.get(name) {
                for entry in maps {
                    if entry.acl_required {
                        let has_acl = self.anti_corruption_layers.iter().any(|a| {
                            a.source_context == entry.source && a.target_context == entry.target
                        });
                        if !has_acl {
                            analysis.missing_acls.push(format!(
                                "Missing ACL between '{}' and '{}'",
                                entry.source, entry.target
                            ));
                        }
                    }
                }
            }
        }

        analysis
    }
}

#[derive(Debug)]
pub struct BoundaryAnalysis {
    pub context_name: String,
    pub incoming_dependencies: Vec<String>,
    pub outgoing_dependencies: Vec<String>,
    pub violations: Vec<String>,
    pub missing_acls: Vec<String>,
}

impl Default for ArchitectureRegistry {
    fn default() -> Self {
        Self::new()
    }
}
