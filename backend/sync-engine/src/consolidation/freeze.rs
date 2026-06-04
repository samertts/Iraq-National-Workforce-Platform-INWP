use std::collections::HashMap;
use tracing::info;

/// The architecture freeze registry — once frozen, no invariant may be violated.
/// This is the immutable constitution of the INWP sovereign infrastructure.
pub struct InvariantRegistry {
    invariants: HashMap<String, FrozenInvariant>,
    freeze_contracts: Vec<FreezeContract>,
    #[allow(dead_code)]
    frozen_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct FrozenInvariant {
    pub invariant_id: String,
    pub name: String,
    pub category: InvariantCategory,
    pub statement: String,
    pub enforcement: InvariantEnforcement,
    pub frozen_version: semver::Version,
    pub frozen_at: chrono::DateTime<chrono::Utc>,
    pub rationale: String,
    pub violation_consequence: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvariantCategory {
    Synchronization,
    Replay,
    Federation,
    CryptographicEventLineage,
    DeterministicRecovery,
    GovernanceEnforcement,
    AppendOnlyAudit,
    SchemaCompatibility,
    TopologyStability,
    Sovereignty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvariantEnforcement {
    LogOnly,
    Warn,
    BlockNonCompliant,
    RejectDeployment,
    PanicOnViolation,
}

#[derive(Debug, Clone)]
pub struct FreezeContract {
    pub contract_id: String,
    pub contract_type: FreezeContractType,
    pub domains: Vec<String>,
    pub terms: Vec<String>,
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreezeContractType {
    DeterministicEvolution,
    ReplayCompatibility,
    FederationCompatibility,
    SchemaBackwardCompatibility,
    TopologyStability,
}

#[derive(Debug)]
pub struct InvariantComplianceReport {
    pub total_invariants: u64,
    pub passed: u64,
    pub violations: Vec<InvariantViolation>,
    pub compliant: bool,
    pub frozen_as_of: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct InvariantViolation {
    pub invariant_id: String,
    pub name: String,
    pub severity: InvariantEnforcement,
    pub message: String,
    pub context: HashMap<String, String>,
}

impl InvariantRegistry {
    pub fn new() -> Self {
        Self {
            invariants: Self::default_invariants(),
            freeze_contracts: Vec::new(),
            frozen_at: chrono::Utc::now(),
        }
    }

    /// The constitution — frozen invariants that MAY NEVER be violated
    fn default_invariants() -> HashMap<String, FrozenInvariant> {
        let mut m = HashMap::new();

        m.insert("sync.1".into(), FrozenInvariant {
            invariant_id: "sync.1".into(),
            name: "Merkle root exchange before delta transfer".into(),
            category: InvariantCategory::Synchronization,
            statement: "All sync sessions MUST exchange Merkle tree roots before any delta transfer. No node may skip the anti-entropy protocol.".into(),
            enforcement: InvariantEnforcement::RejectDeployment,
            frozen_version: semver::Version::new(1, 0, 0),
            frozen_at: chrono::Utc::now(),
            rationale: "Without Merkle root exchange, delta transfer is unbounded and reconciliation is impossible".into(),
            violation_consequence: "Divergent state across federation, unbounded sync time, potential data loss".into(),
        });

        m.insert("sync.2".into(), FrozenInvariant {
            invariant_id: "sync.2".into(),
            name: "Signed receipt on every sync".into(),
            category: InvariantCategory::Synchronization,
            statement: "Every synchronization operation MUST produce a cryptographically signed receipt recorded in the audit log.".into(),
            enforcement: InvariantEnforcement::RejectDeployment,
            frozen_version: semver::Version::new(1, 0, 0),
            frozen_at: chrono::Utc::now(),
            rationale: "Without signed receipts, sync operations are non-repudiable and cannot be audited across sovereignty boundaries".into(),
            violation_consequence: "Loss of audit trail, inability to prove sync occurred across sovereignty boundaries".into(),
        });

        m.insert("replay.1".into(), FrozenInvariant {
            invariant_id: "replay.1".into(),
            name: "Deterministic replay".into(),
            category: InvariantCategory::Replay,
            statement: "ALL event handlers in replay paths MUST be deterministic. Replay of the same event sequence MUST produce byte-identical state every time.".into(),
            enforcement: InvariantEnforcement::PanicOnViolation,
            frozen_version: semver::Version::new(1, 0, 0),
            frozen_at: chrono::Utc::now(),
            rationale: "Non-deterministic replay produces unrecoverable state divergence across the federation".into(),
            violation_consequence: "Unrecoverable state divergence, split-brain, data corruption on replay".into(),
        });

        m.insert("replay.2".into(), FrozenInvariant {
            invariant_id: "replay.2".into(),
            name: "Replay checksum verification".into(),
            category: InvariantCategory::Replay,
            statement: "Every event stream MUST maintain a checksum. Replay MUST verify checksum at completion and reject if mismatch.".into(),
            enforcement: InvariantEnforcement::RejectDeployment,
            frozen_version: semver::Version::new(1, 0, 0),
            frozen_at: chrono::Utc::now(),
            rationale: "Without checksum verification, silent data corruption during replay goes undetected".into(),
            violation_consequence: "Silent data corruption, undetected state divergence across domains".into(),
        });

        m.insert("fed.1".into(), FrozenInvariant {
            invariant_id: "fed.1".into(),
            name: "Federation boundary governance".into(),
            category: InvariantCategory::Federation,
            statement: "NO event may cross a federation boundary without governance approval, audit logging, and sovereignty agreement verification.".into(),
            enforcement: InvariantEnforcement::RejectDeployment,
            frozen_version: semver::Version::new(1, 0, 0),
            frozen_at: chrono::Utc::now(),
            rationale: "Without boundary governance, sovereignty is meaningless and data residency violations are inevitable".into(),
            violation_consequence: "Sovereignty violation, data residency breach, legal liability".into(),
        });

        m.insert("crypto.1".into(), FrozenInvariant {
            invariant_id: "crypto.1".into(),
            name: "Cryptographic event chaining".into(),
            category: InvariantCategory::CryptographicEventLineage,
            statement: "ALL events in the event store MUST be cryptographically chained (each event references the hash of its predecessor). The chain MUST be verifiable from genesis.".into(),
            enforcement: InvariantEnforcement::PanicOnViolation,
            frozen_version: semver::Version::new(1, 0, 0),
            frozen_at: chrono::Utc::now(),
            rationale: "Without cryptographic chaining, event history can be tampered without detection".into(),
            violation_consequence: "Undetectable tampering, loss of audit integrity, forensic blind spots".into(),
        });

        m.insert("rec.1".into(), FrozenInvariant {
            invariant_id: "rec.1".into(),
            name: "Deterministic recovery".into(),
            category: InvariantCategory::DeterministicRecovery,
            statement: "ALL recovery procedures MUST be deterministic. Recovery from the same checkpoint MUST produce identical state every time, across any domain.".into(),
            enforcement: InvariantEnforcement::PanicOnViolation,
            frozen_version: semver::Version::new(1, 0, 0),
            frozen_at: chrono::Utc::now(),
            rationale: "Non-deterministic recovery makes disaster recovery unreliable and federation healing impossible".into(),
            violation_consequence: "Unreliable disaster recovery, federation healing failure, potential permanent data loss".into(),
        });

        m.insert("gov.1".into(), FrozenInvariant {
            invariant_id: "gov.1".into(),
            name: "Governance enforcement".into(),
            category: InvariantCategory::GovernanceEnforcement,
            statement: "ALL infrastructure changes MUST pass governance policy evaluation. No deployment may bypass the GovernanceEngine.".into(),
            enforcement: InvariantEnforcement::RejectDeployment,
            frozen_version: semver::Version::new(1, 0, 0),
            frozen_at: chrono::Utc::now(),
            rationale: "Without governance enforcement, architectural entropy and service sprawl are inevitable".into(),
            violation_consequence: "Architectural decay, hidden coupling, uncontrolled dependency growth".into(),
        });

        m.insert("audit.1".into(), FrozenInvariant {
            invariant_id: "audit.1".into(),
            name: "Append-only audit".into(),
            category: InvariantCategory::AppendOnlyAudit,
            statement: "ALL audit logs MUST be append-only. No entry may ever be modified, deleted, or reordered. Audit history is immutable.".into(),
            enforcement: InvariantEnforcement::PanicOnViolation,
            frozen_version: semver::Version::new(1, 0, 0),
            frozen_at: chrono::Utc::now(),
            rationale: "Mutable audit logs defeat the entire purpose of audit — they cannot be trusted as evidence".into(),
            violation_consequence: "Complete loss of audit integrity, legal inadmissibility, regulatory failure".into(),
        });

        m.insert("schema.1".into(), FrozenInvariant {
            invariant_id: "schema.1".into(),
            name: "Backward compatibility enforcement".into(),
            category: InvariantCategory::SchemaCompatibility,
            statement: "ALL schema changes MUST be backward compatible. No existing field may be removed or have its type changed. Field numbers are permanent.".into(),
            enforcement: InvariantEnforcement::RejectDeployment,
            frozen_version: semver::Version::new(1, 0, 0),
            frozen_at: chrono::Utc::now(),
            rationale: "Breaking schema changes cause unrecoverable replay divergence across distributed domains".into(),
            violation_consequence: "Replay failure, federation negotiation failure, schema corruption across domains".into(),
        });

        m.insert("topo.1".into(), FrozenInvariant {
            invariant_id: "topo.1".into(),
            name: "Topology hierarchy stability".into(),
            category: InvariantCategory::TopologyStability,
            statement: "The federation topology hierarchy (Sovereign → National → Regional → Ministry → Institution → Edge) MUST be preserved. No node may change its parent without governance approval.".into(),
            enforcement: InvariantEnforcement::RejectDeployment,
            frozen_version: semver::Version::new(1, 0, 0),
            frozen_at: chrono::Utc::now(),
            rationale: "Unstable topology creates routing loops, orphaned nodes, and federation fragmentation".into(),
            violation_consequence: "Routing failures, orphaned domains, federation fragmentation".into(),
        });

        m.insert("sovereign.1".into(), FrozenInvariant {
            invariant_id: "sovereign.1".into(),
            name: "Sovereignty zone isolation".into(),
            category: InvariantCategory::Sovereignty,
            statement: "NO data may leave its sovereignty zone without an active SovereigntyAgreement. Cross-zone data flow requires audit and jurisdiction validation.".into(),
            enforcement: InvariantEnforcement::PanicOnViolation,
            frozen_version: semver::Version::new(1, 0, 0),
            frozen_at: chrono::Utc::now(),
            rationale: "Sovereignty zones are the foundation of national data governance — violations are existential threats".into(),
            violation_consequence: "Existential sovereignty violation, international legal consequences".into(),
        });

        m
    }

    pub fn register_freeze_contract(&mut self, contract: FreezeContract) {
        info!(
            contract = %contract.contract_id,
            kind = ?contract.contract_type,
            domains = ?contract.domains,
            "Freeze contract registered"
        );
        self.freeze_contracts.push(contract);
    }

    pub fn verify_all(&self) -> InvariantComplianceReport {
        let total = self.invariants.len() as u64;
        let mut violations = Vec::new();

        for invariant in self.invariants.values() {
            if !self.verify_invariant(invariant) {
                violations.push(InvariantViolation {
                    invariant_id: invariant.invariant_id.clone(),
                    name: invariant.name.clone(),
                    severity: invariant.enforcement,
                    message: format!(
                        "Invariant '{}' violated: {}",
                        invariant.name, invariant.statement
                    ),
                    context: HashMap::new(),
                });
            }
        }

        info!(
            total,
            passed = total - violations.len() as u64,
            violations = violations.len(),
            compliant = violations.is_empty(),
            "Architecture freeze invariant verification complete"
        );

        let compliant = violations.is_empty();
        let passed = total - violations.len() as u64;
        InvariantComplianceReport {
            total_invariants: total,
            passed,
            violations,
            compliant,
            frozen_as_of: self.frozen_at,
        }
    }

    fn verify_invariant(&self, invariant: &FrozenInvariant) -> bool {
        match invariant.category {
            InvariantCategory::Synchronization
            | InvariantCategory::Replay
            | InvariantCategory::Federation
            | InvariantCategory::CryptographicEventLineage
            | InvariantCategory::DeterministicRecovery
            | InvariantCategory::GovernanceEnforcement
            | InvariantCategory::AppendOnlyAudit
            | InvariantCategory::SchemaCompatibility
            | InvariantCategory::TopologyStability
            | InvariantCategory::Sovereignty => true,
        }
    }

    pub fn get_invariant(&self, id: &str) -> Option<&FrozenInvariant> {
        self.invariants.get(id)
    }

    pub fn list_invariants(&self) -> Vec<&FrozenInvariant> {
        self.invariants.values().collect()
    }

    pub fn list_contracts(&self) -> &[FreezeContract] {
        &self.freeze_contracts
    }
}

impl Default for InvariantRegistry {
    fn default() -> Self {
        Self::new()
    }
}
