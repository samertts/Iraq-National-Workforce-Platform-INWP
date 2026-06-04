use std::collections::HashSet;
use tracing::info;

/// Compatibility analysis for schema evolution, replay safety, and federation negotiation
pub struct CompatibilityValidator {
    known_transforms: Vec<SchemaTransform>,
    compatibility_rules: Vec<CompatibilityRule>,
}

#[derive(Debug, Clone)]
pub struct SchemaTransform {
    pub from_schema: String,
    pub to_schema: String,
    pub transform_type: TransformType,
    pub backward_compatible: bool,
    pub forward_compatible: bool,
    pub replay_safe: bool,
    pub corruption_risk: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformType {
    AddField,
    RemoveField,
    RenameField,
    ChangeType,
    MakeOptional,
    MakeRequired,
    Restructure,
    Split,
    Merge,
}

#[derive(Debug, Clone)]
pub struct CompatibilityRule {
    pub rule_name: String,
    pub allow_removal: bool,
    pub allow_type_change: bool,
    pub allow_rename: bool,
    pub max_field_number_collisions: usize,
    pub require_reserved_numbers: bool,
    pub require_field_preservation: Vec<String>,
}

#[derive(Debug)]
pub struct CompatibilityReport {
    pub from_schema: String,
    pub to_schema: String,
    pub backward_compatible: bool,
    pub forward_compatible: bool,
    pub replay_compatible: bool,
    pub breaking_changes: Vec<BreakingChange>,
    pub warnings: Vec<String>,
    pub recommendation: MigrationRecommendation,
}

#[derive(Debug)]
pub struct BreakingChange {
    pub change_type: TransformType,
    pub field: String,
    pub impact: ImpactLevel,
    pub description: String,
    pub mitigation: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationRecommendation {
    Safe,
    RequiresMigration,
    RequiresCoordinatedDeployment,
    RequiresFederationNegotiation,
    Blocked,
}

impl CompatibilityValidator {
    pub fn new() -> Self {
        Self {
            known_transforms: Vec::new(),
            compatibility_rules: Vec::new(),
        }
    }

    pub fn register_transform(&mut self, transform: SchemaTransform) {
        info!(
            from = %transform.from_schema,
            to = %transform.to_schema,
            compatible = %transform.backward_compatible,
            "Schema transform registered"
        );
        self.known_transforms.push(transform);
    }

    pub fn add_rule(&mut self, rule: CompatibilityRule) {
        info!(rule = %rule.rule_name, "Compatibility rule added");
        self.compatibility_rules.push(rule);
    }

    pub fn analyze(&self, from_schema: &str, to_schema: &str) -> CompatibilityReport {
        let mut breaking_changes = Vec::new();
        let mut warnings = Vec::new();

        let matching_transforms: Vec<&SchemaTransform> = self
            .known_transforms
            .iter()
            .filter(|t| t.from_schema == from_schema && t.to_schema == to_schema)
            .collect();

        let backward = matching_transforms.iter().all(|t| t.backward_compatible);
        let forward = matching_transforms.iter().all(|t| t.forward_compatible);
        let replay = matching_transforms.iter().all(|t| t.replay_safe);

        if !backward {
            let m = matching_transforms.first().unwrap();
            breaking_changes.push(BreakingChange {
                change_type: m.transform_type,
                field: format!("{} -> {}", from_schema, to_schema),
                impact: ImpactLevel::High,
                description: format!("Schema transform from '{}' to '{}' breaks backward compatibility", from_schema, to_schema),
                mitigation: if m.corruption_risk > 0.5 {
                    Some("Add data migration layer; schema change may corrupt existing events on replay".into())
                } else {
                    Some("Deploy schema change as optional field addition".into())
                },
            });
        }

        if !forward {
            warnings.push(format!(
                "Forward compatibility not guaranteed for '{}' -> '{}'",
                from_schema, to_schema
            ));
        }

        if !replay {
            breaking_changes.push(BreakingChange {
                change_type: matching_transforms.first().map(|t| t.transform_type).unwrap_or(TransformType::Restructure),
                field: "replay".into(),
                impact: ImpactLevel::Critical,
                description: format!(
                    "Schema transform from '{}' to '{}' is not replay-safe — replaying old events will produce divergent state",
                    from_schema, to_schema
                ),
                mitigation: Some("Ensure deterministic deserialization for all historical schema versions".into()),
            });
        }

        let recommendation = if backward && replay {
            MigrationRecommendation::Safe
        } else if !backward {
            MigrationRecommendation::Blocked
        } else if !replay {
            MigrationRecommendation::RequiresMigration
        } else {
            MigrationRecommendation::RequiresCoordinatedDeployment
        };

        CompatibilityReport {
            from_schema: from_schema.to_string(),
            to_schema: to_schema.to_string(),
            backward_compatible: backward,
            forward_compatible: forward,
            replay_compatible: replay,
            breaking_changes,
            warnings,
            recommendation,
        }
    }

    pub fn validate_federation_compatibility(
        &self,
        local_schemas: &[String],
        remote_schemas: &[String],
    ) -> FederationCompatibilityReport {
        let mut incompatibilities = Vec::new();
        let local_set: HashSet<&String> = local_schemas.iter().collect();
        let remote_set: HashSet<&String> = remote_schemas.iter().collect();

        for schema in local_set.intersection(&remote_set) {
            let report = self.analyze(schema, schema);
            if !report.backward_compatible {
                incompatibilities.push(format!(
                    "Schema '{}' has incompatible versions across federation boundary",
                    schema
                ));
            }
        }

        for schema in local_set.difference(&remote_set) {
            incompatibilities.push(format!(
                "Schema '{}' present locally but missing from remote federation",
                schema
            ));
        }

        for schema in remote_set.difference(&local_set) {
            incompatibilities.push(format!(
                "Schema '{}' present on remote but missing locally",
                schema
            ));
        }

        let compatible = incompatibilities.is_empty();
        FederationCompatibilityReport {
            local_schemas: local_schemas.len(),
            remote_schemas: remote_schemas.len(),
            intersection: local_set.intersection(&remote_set).count(),
            incompatibilities,
            compatible,
        }
    }
}

#[derive(Debug)]
pub struct FederationCompatibilityReport {
    pub local_schemas: usize,
    pub remote_schemas: usize,
    pub intersection: usize,
    pub incompatibilities: Vec<String>,
    pub compatible: bool,
}

impl Default for CompatibilityValidator {
    fn default() -> Self {
        Self::new()
    }
}
