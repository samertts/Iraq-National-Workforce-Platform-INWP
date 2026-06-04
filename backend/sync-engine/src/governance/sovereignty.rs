use super::{GovernanceViolation, PolicyEvalResult, PolicySeverity};
use std::collections::HashMap;
use tracing::info;

/// Registry for sovereignty zones, data residency policies, and jurisdiction mappings
pub struct SovereigntyRegistry {
    zones: HashMap<String, SovereigntyZone>,
    residency_policies: Vec<ResidencyPolicy>,
    jurisdiction_map: HashMap<String, Jurisdiction>,
    partition_policies: Vec<PartitionPolicy>,
    zone_hierarchy: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SovereigntyZone {
    pub zone_id: String,
    pub name: String,
    pub zone_type: ZoneType,
    pub jurisdiction: String,
    pub parent_zone: Option<String>,
    pub data_classifications: Vec<DataClassification>,
    pub allowed_regions: Vec<String>,
    pub restricted_regions: Vec<String>,
    pub isolation_level: IsolationLevel,
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    Sovereign,
    National,
    Ministry,
    Regional,
    Edge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Secret,
    TopSecret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolationLevel {
    Logical,
    Physical,
    Cryptographic,
    AirGapped,
}

#[derive(Debug, Clone)]
pub struct ResidencyPolicy {
    pub policy_id: String,
    pub data_classification: DataClassification,
    pub allowed_zones: Vec<String>,
    pub forbidden_zones: Vec<String>,
    pub require_jurisdiction_compliance: bool,
    pub retention_days: u64,
    pub audit_required: bool,
}

#[derive(Debug, Clone)]
pub struct Jurisdiction {
    pub jurisdiction_id: String,
    pub name: String,
    pub country: String,
    pub regulatory_framework: String,
    pub data_protection_law: String,
    pub requires_local_storage: bool,
    pub requires_cross_border_agreement: bool,
}

#[derive(Debug, Clone)]
pub struct PartitionPolicy {
    pub policy_id: String,
    pub zone_id: String,
    pub partition_key: String,
    pub replication_factor: u32,
    /// domains that may receive this partition's data
    pub allowed_replication_targets: Vec<String>,
    pub encrypted: bool,
    pub compressed: bool,
}

#[derive(Debug)]
pub struct SovereigntyComplianceReport {
    pub zone: String,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub residency_checks_passed: usize,
    pub residency_checks_failed: usize,
}

impl SovereigntyRegistry {
    pub fn new() -> Self {
        Self {
            zones: HashMap::new(),
            residency_policies: Vec::new(),
            jurisdiction_map: HashMap::new(),
            partition_policies: Vec::new(),
            zone_hierarchy: HashMap::new(),
        }
    }

    pub fn register_zone(&mut self, zone: SovereigntyZone) {
        let id = zone.zone_id.clone();
        if let Some(parent) = &zone.parent_zone {
            self.zone_hierarchy
                .entry(parent.clone())
                .or_default()
                .push(id.clone());
        }
        info!(zone = %id, kind = ?zone.zone_type, "Sovereignty zone registered");
        self.zones.insert(id, zone);
    }

    pub fn register_residency_policy(&mut self, policy: ResidencyPolicy) {
        info!(
            classification = ?policy.data_classification,
            "Data residency policy registered"
        );
        self.residency_policies.push(policy);
    }

    pub fn register_jurisdiction(&mut self, jurisdiction: Jurisdiction) {
        let id = jurisdiction.jurisdiction_id.clone();
        info!(jurisdiction = %id, country = %jurisdiction.country, "Jurisdiction registered");
        self.jurisdiction_map.insert(id, jurisdiction);
    }

    pub fn register_partition_policy(&mut self, policy: PartitionPolicy) {
        info!(
            zone = %policy.zone_id,
            partition = %policy.partition_key,
            "Partition policy registered"
        );
        self.partition_policies.push(policy);
    }

    pub fn check_residency(&self, zone_id: &str, allowed_regions: &[String]) -> PolicyEvalResult {
        let zone = match self.zones.get(zone_id) {
            Some(z) => z,
            None => {
                return PolicyEvalResult::Violation(GovernanceViolation {
                    policy_id: uuid::Uuid::nil(),
                    policy_name: "SovereigntyResidency".into(),
                    severity: PolicySeverity::Critical,
                    message: format!("Sovereignty zone '{}' not registered", zone_id),
                    context: {
                        let mut m = HashMap::new();
                        m.insert("zone_id".into(), zone_id.into());
                        m
                    },
                    remediations: vec![format!("Register sovereignty zone '{}'", zone_id)],
                })
            }
        };

        for region in allowed_regions {
            if zone.restricted_regions.contains(region) {
                return PolicyEvalResult::Violation(GovernanceViolation {
                    policy_id: uuid::Uuid::nil(),
                    policy_name: "SovereigntyResidency".into(),
                    severity: PolicySeverity::Error,
                    message: format!(
                        "Region '{}' is restricted for sovereignty zone '{}'",
                        region, zone_id
                    ),
                    context: {
                        let mut m = HashMap::new();
                        m.insert("zone_id".into(), zone_id.into());
                        m.insert("region".into(), region.into());
                        m
                    },
                    remediations: vec![
                        "Choose a different region for data residency".into(),
                        "Update restricted regions list for this zone".into(),
                    ],
                });
            }
        }

        PolicyEvalResult::Pass
    }

    pub fn check_data_residency_compliance(
        &self,
        zone_id: &str,
        classification: DataClassification,
    ) -> SovereigntyComplianceReport {
        let mut violations = Vec::new();
        let mut checks_passed = 0;
        let mut checks_failed = 0;

        let zone = self.zones.get(zone_id);
        let applicable_policies: Vec<&ResidencyPolicy> = self
            .residency_policies
            .iter()
            .filter(|p| p.data_classification == classification)
            .collect();

        for policy in &applicable_policies {
            let in_allowed = policy.allowed_zones.is_empty()
                || zone
                    .map(|z| policy.allowed_zones.contains(&z.zone_id))
                    .unwrap_or(false);
            let in_forbidden = zone
                .map(|z| policy.forbidden_zones.contains(&z.zone_id))
                .unwrap_or(false);

            if in_forbidden {
                violations.push(format!(
                    "Data classification {:?} is forbidden in zone '{}' by policy '{}'",
                    classification, zone_id, policy.policy_id
                ));
                checks_failed += 1;
            } else if !in_allowed && !policy.allowed_zones.is_empty() {
                violations.push(format!(
                    "Zone '{}' not in allowed zones for data classification {:?}",
                    zone_id, classification
                ));
                checks_failed += 1;
            } else {
                checks_passed += 1;
            }
        }

        SovereigntyComplianceReport {
            zone: zone_id.to_string(),
            compliant: violations.is_empty(),
            violations,
            residency_checks_passed: checks_passed,
            residency_checks_failed: checks_failed,
        }
    }

    pub fn get_zone(&self, zone_id: &str) -> Option<&SovereigntyZone> {
        self.zones.get(zone_id)
    }

    pub fn list_zones(&self) -> Vec<&SovereigntyZone> {
        self.zones.values().collect()
    }

    pub fn list_partition_policies(&self) -> &[PartitionPolicy] {
        &self.partition_policies
    }

    pub fn get_child_zones(&self, parent_id: &str) -> Vec<&SovereigntyZone> {
        self.zone_hierarchy
            .get(parent_id)
            .map(|children| {
                children
                    .iter()
                    .filter_map(|id| self.zones.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn validate_partition_replication(
        &self,
        partition_key: &str,
        target_domain: &str,
    ) -> PolicyEvalResult {
        for policy in &self.partition_policies {
            if policy.partition_key == partition_key {
                if !policy
                    .allowed_replication_targets
                    .iter()
                    .any(|t| t == target_domain)
                {
                    return PolicyEvalResult::Violation(GovernanceViolation {
                        policy_id: uuid::Uuid::nil(),
                        policy_name: "SovereigntyResidency".into(),
                        severity: PolicySeverity::Error,
                        message: format!(
                            "Partition '{}' not allowed to replicate to domain '{}'",
                            partition_key, target_domain
                        ),
                        context: {
                            let mut m = HashMap::new();
                            m.insert("partition_key".into(), partition_key.into());
                            m.insert("target_domain".into(), target_domain.into());
                            m
                        },
                        remediations: vec![
                            "Add domain to allowed replication targets".into(),
                            "Choose a different replication target".into(),
                        ],
                    });
                }
                return PolicyEvalResult::Pass;
            }
        }
        PolicyEvalResult::Pass
    }
}

impl Default for SovereigntyRegistry {
    fn default() -> Self {
        Self::new()
    }
}
