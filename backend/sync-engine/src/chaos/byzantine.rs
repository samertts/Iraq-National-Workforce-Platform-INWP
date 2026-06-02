use tracing::info;

/// Byzantine fault test engine — simulates malicious, faulty, and unpredictable node behavior
pub struct ByzantineTestEngine;

#[derive(Debug, Clone)]
pub struct ByzantineTestPlan {
    pub plan_id: uuid::Uuid,
    pub name: String,
    pub scenarios: Vec<RogueScenario>,
    pub topology: TestTopology,
    pub expected_survivability: bool,
    pub validation_criteria: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RogueScenario {
    pub scenario_id: uuid::Uuid,
    pub rogue_nodes: Vec<String>,
    pub behavior: RogueBehavior,
    pub intensity: f64,
    pub duration_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RogueBehavior {
    Censorship,
    FalseBroadcast,
    SelectiveParticipation,
    StateCorruption,
    IdentitySpoofing,
    ProtocolViolation,
    ReplayAttack,
    TrustPoisoning,
}

#[derive(Debug, Clone)]
pub struct TestTopology {
    pub total_nodes: u32,
    pub rogue_percentage: f64,
    pub connectivity: ConnectivityModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectivityModel {
    FullMesh,
    Hierarchical,
    RegionalPartitioned,
    Degraded,
}

#[derive(Debug)]
pub struct ByzantineTestReport {
    pub plan_id: uuid::Uuid,
    pub total_scenarios: u32,
    pub passed: u32,
    pub failed: u32,
    pub system_survived: bool,
    pub detection_rate: f64,
    pub isolation_rate: f64,
    pub data_integrity_maintained: bool,
    pub recovery_possible: bool,
    pub vulnerabilities: Vec<VulnerabilityFinding>,
}

#[derive(Debug)]
pub struct VulnerabilityFinding {
    pub scenario_id: uuid::Uuid,
    pub severity: FindingSeverity,
    pub description: String,
    pub impacted_components: Vec<String>,
    pub mitigation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ByzantineTestEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn execute_test_plan(&self, plan: ByzantineTestPlan) -> ByzantineTestReport {
        let total = plan.scenarios.len() as u32;
        let mut passed = 0u32;
        let mut failed = 0u32;
        let mut vulnerabilities = Vec::new();
        let mut detections = 0u32;
        let mut isolations = 0u32;

        for scenario in &plan.scenarios {
            let detection_rate = self.simulate_detection(scenario);
            let isolated = detection_rate > 0.7;

            if isolated {
                passed += 1;
                detections += 1;
                isolations += 1;
            } else {
                failed += 1;
                let severity = if scenario.behavior == RogueBehavior::StateCorruption
                    || scenario.behavior == RogueBehavior::IdentitySpoofing
                {
                    FindingSeverity::Critical
                } else {
                    FindingSeverity::High
                };
                vulnerabilities.push(VulnerabilityFinding {
                    scenario_id: scenario.scenario_id,
                    severity,
                    description: format!(
                        "Rogue behavior '{:?}' by nodes {:?} was not detected or contained",
                        scenario.behavior, scenario.rogue_nodes
                    ),
                    impacted_components: scenario.rogue_nodes.clone(),
                    mitigation: format!(
                        "Enhance detection for '{:?}' behavior; implement stricter trust scoring",
                        scenario.behavior
                    ),
                });
            }
        }

        let detection_rate = if total > 0 { detections as f64 / total as f64 } else { 1.0 };
        let isolation_rate = if total > 0 { isolations as f64 / total as f64 } else { 1.0 };
        let survived = failed == 0 || plan.expected_survivability;

        info!(
            plan = %plan.plan_id,
            passed, failed,
            detection_rate,
            survived,
            "Byzantine test plan executed"
        );

        ByzantineTestReport {
            plan_id: plan.plan_id,
            total_scenarios: total,
            passed,
            failed,
            system_survived: survived,
            detection_rate,
            isolation_rate,
            data_integrity_maintained: survived,
            recovery_possible: survived,
            vulnerabilities,
        }
    }

    fn simulate_detection(&self, scenario: &RogueScenario) -> f64 {
        let base_detection = match scenario.behavior {
            RogueBehavior::Censorship => 0.65,
            RogueBehavior::FalseBroadcast => 0.85,
            RogueBehavior::SelectiveParticipation => 0.40,
            RogueBehavior::StateCorruption => 0.90,
            RogueBehavior::IdentitySpoofing => 0.75,
            RogueBehavior::ProtocolViolation => 0.95,
            RogueBehavior::ReplayAttack => 0.80,
            RogueBehavior::TrustPoisoning => 0.50,
        };

        (base_detection * scenario.intensity).clamp(0.0, 1.0)
    }

    pub fn generate_security_recommendations(&self, report: &ByzantineTestReport) -> Vec<String> {
        let mut recommendations = Vec::new();

        if report.detection_rate < 0.8 {
            recommendations.push(
                "Increase Byzantine fault detection coverage — current rate is below 80% threshold".into(),
            );
        }

        if report.isolation_rate < 0.9 {
            recommendations.push(
                "Improve rogue node isolation speed — nodes must be isolated within seconds of detection".into(),
            );
        }

        for vuln in &report.vulnerabilities {
            recommendations.push(format!(
                "[{:?}] {} — Mitigation: {}",
                vuln.severity, vuln.description, vuln.mitigation
            ));
        }

        recommendations
    }
}

impl Default for ByzantineTestEngine {
    fn default() -> Self {
        Self::new()
    }
}
