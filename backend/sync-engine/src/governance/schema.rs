use super::{GovernanceViolation, PolicyEvalResult, PolicySeverity};
use crate::error::SyncResult;
use std::collections::HashMap;
use tracing::info;

/// Governance for all protobuf/event schemas in the sovereign architecture
pub struct SchemaGovernance {
    contracts: HashMap<String, SchemaContract>,
    compatibility_policies: Vec<CompatibilityPolicy>,
    evolution_rules: Vec<EvolutionRuleDefinition>,
    deprecated_schemas: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SchemaContract {
    pub contract_id: String,
    pub name: String,
    pub version: semver::Version,
    pub schema_type: SchemaType,
    pub domain: String,
    pub owner: String,
    pub fields: Vec<SchemaField>,
    pub status: SchemaStatus,
    pub fingerprint: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub checksum: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaType {
    Event,
    Command,
    Query,
    State,
    SyncContract,
    FederationContract,
}

#[derive(Debug, Clone)]
pub struct SchemaField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub deprecated: bool,
    pub description: String,
    pub constraints: Vec<String>,
    pub removed_in_version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaStatus {
    Draft,
    Active,
    Deprecated,
    Sunset,
    Retired,
}

#[derive(Debug, Clone)]
pub struct CompatibilityPolicy {
    pub contract_id: String,
    pub backward_compatible: bool,
    pub forward_compatible: bool,
    pub require_field_preservation: bool,
    pub max_field_removals: usize,
    pub require_idempotency: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvolutionRule {
    AllowAddField,
    AllowRemoveField,
    AllowRenameField,
    AllowChangeType,
    AllowMakeOptional,
    AllowMakeRequired,
    RequireFieldPreservation(String),
    RequireBackwardCompatibility,
    RequireForwardCompatibility,
}

#[derive(Debug, Clone)]
pub struct EvolutionRuleDefinition {
    pub contract_id: String,
    pub rules: Vec<EvolutionRule>,
}

#[derive(Debug)]
pub struct CompatibilityReport {
    pub contract_id: String,
    pub source_version: String,
    pub target_version: String,
    pub backward_compatible: bool,
    pub forward_compatible: bool,
    pub violations: Vec<String>,
    pub breaking_changes: Vec<String>,
}

impl SchemaGovernance {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
            compatibility_policies: Vec::new(),
            evolution_rules: Vec::new(),
            deprecated_schemas: Vec::new(),
        }
    }

    pub fn register_contract(&mut self, contract: SchemaContract) {
        let id = contract.contract_id.clone();
        info!(
            contract = %id,
            version = %contract.version,
            domain = %contract.domain,
            "Schema contract registered"
        );
        self.contracts.insert(id, contract);
    }

    pub fn register_compatibility_policy(&mut self, policy: CompatibilityPolicy) {
        let id = policy.contract_id.clone();
        info!(contract = %id, "Compatibility policy registered");
        self.compatibility_policies.push(policy);
    }

    pub fn register_evolution_rule(&mut self, rule: EvolutionRuleDefinition) {
        let id = rule.contract_id.clone();
        info!(contract = %id, "Evolution rules registered");
        self.evolution_rules.push(rule);
    }

    pub fn validate_evolution(
        &self,
        contract_id: &str,
        old_version: &str,
        new_version: &str,
    ) -> CompatibilityReport {
        let mut violations = Vec::new();
        let mut breaking_changes = Vec::new();

        let old = self.contracts.get(contract_id);
        let policy = self
            .compatibility_policies
            .iter()
            .find(|p| p.contract_id == contract_id);

        if let Some(policy) = policy {
            if policy.backward_compatible {
                if let Some(old_contract) = old {
                    for field in &old_contract.fields {
                        if field.removed_in_version.as_deref() == Some(new_version)
                            && policy.require_field_preservation
                        {
                            violations.push(format!(
                                "Requires backward compatibility but field '{}' was removed",
                                field.name
                            ));
                            breaking_changes.push(format!("Removed field '{}'", field.name));
                        }
                    }
                }
            }
        }

        CompatibilityReport {
            contract_id: contract_id.to_string(),
            source_version: old_version.to_string(),
            target_version: new_version.to_string(),
            backward_compatible: violations.is_empty(),
            forward_compatible: violations.is_empty(),
            violations,
            breaking_changes,
        }
    }

    pub fn check_evolution_rule(
        &self,
        contract_id: &str,
        rule: &EvolutionRule,
    ) -> PolicyEvalResult {
        for ruleset in &self.evolution_rules {
            if ruleset.contract_id == contract_id && !ruleset.rules.contains(rule) {
                return PolicyEvalResult::Violation(GovernanceViolation {
                    policy_id: uuid::Uuid::nil(),
                    policy_name: "SchemaEvolutionRule".into(),
                    severity: PolicySeverity::Error,
                    message: format!(
                        "Evolution rule '{:?}' not allowed for contract '{}'",
                        rule, contract_id
                    ),
                    context: {
                        let mut m = HashMap::new();
                        m.insert("contract_id".into(), contract_id.into());
                        m.insert("rule".into(), format!("{:?}", rule));
                        m
                    },
                    remediations: vec![format!(
                        "Add evolution rule '{:?}' to contract '{}'",
                        rule, contract_id
                    )],
                });
            }
        }
        PolicyEvalResult::Pass
    }

    pub fn deprecate_contract(&mut self, contract_id: &str) -> SyncResult<()> {
        if let Some(contract) = self.contracts.get_mut(contract_id) {
            contract.status = SchemaStatus::Deprecated;
            self.deprecated_schemas.push(contract_id.to_string());
            info!(contract = %contract_id, "Schema contract deprecated");
            Ok(())
        } else {
            Err(crate::error::SyncEngineError::Internal(format!(
                "Schema contract '{}' not found for deprecation",
                contract_id
            )))
        }
    }

    pub fn list_active_contracts(&self) -> Vec<&SchemaContract> {
        self.contracts
            .values()
            .filter(|c| matches!(c.status, SchemaStatus::Active))
            .collect()
    }

    pub fn get_contract(&self, contract_id: &str) -> Option<&SchemaContract> {
        self.contracts.get(contract_id)
    }

    pub fn generate_schema_digest(&self, contract_id: &str) -> Option<String> {
        self.contracts
            .get(contract_id)
            .map(|c| c.fingerprint.clone())
    }
}

impl Default for SchemaGovernance {
    fn default() -> Self {
        Self::new()
    }
}
