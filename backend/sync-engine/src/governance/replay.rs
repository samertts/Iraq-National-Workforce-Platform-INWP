use super::{GovernanceViolation, PolicyEvalResult, PolicySeverity};
use std::collections::HashMap;
use tracing::info;

/// Governance for replay safety and deterministic replay across all streams
pub struct ReplayGovernance {
    stream_registry: HashMap<String, ReplayStream>,
    replay_policies: Vec<ReplayPolicy>,
    verification_records: Vec<ReplayVerification>,
    deterministic_checks: HashMap<String, DeterminismCheck>,
}

#[derive(Debug, Clone)]
pub struct ReplayStream {
    pub stream_id: String,
    pub name: String,
    pub domain: String,
    pub event_types: Vec<String>,
    pub total_events: u64,
    pub last_replay: Option<chrono::DateTime<chrono::Utc>>,
    pub deterministic: bool,
    pub checksum: Vec<u8>,
    pub schema_version: String,
}

#[derive(Debug, Clone)]
pub struct ReplayPolicy {
    pub policy_id: String,
    pub require_determinism: bool,
    pub require_checksum_verification: bool,
    pub max_replay_divergence: f64,
    pub require_event_ordering: bool,
    pub require_causal_consistency: bool,
    pub enforcement: ReplayEnforcement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayEnforcement {
    LogOnly,
    Warn,
    Block,
}

#[derive(Debug, Clone)]
pub struct ReplayVerification {
    pub stream_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub events_replayed: u64,
    pub checksum_match: bool,
    pub determinism_verified: bool,
    pub divergence_detected: f64,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DeterminismCheck {
    pub stream_id: String,
    pub deterministic: bool,
    pub non_determinism_sources: Vec<String>,
    pub last_verified: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct ReplaySafetyReport {
    pub stream_id: String,
    pub safe: bool,
    pub determinism_verified: bool,
    pub checksum_valid: bool,
    pub event_ordering_valid: bool,
    pub causal_consistency: bool,
    pub violations: Vec<String>,
}

impl ReplayGovernance {
    pub fn new() -> Self {
        Self {
            stream_registry: HashMap::new(),
            replay_policies: Vec::new(),
            verification_records: Vec::new(),
            deterministic_checks: HashMap::new(),
        }
    }

    pub fn register_stream(&mut self, stream: ReplayStream) {
        let id = stream.stream_id.clone();
        info!(stream = %id, domain = %stream.domain, "Replay stream registered");
        self.stream_registry.insert(id, stream);
    }

    pub fn register_policy(&mut self, policy: ReplayPolicy) {
        info!(policy = %policy.policy_id, "Replay governance policy registered");
        self.replay_policies.push(policy);
    }

    pub fn check_determinism(&self, stream_id: &str) -> PolicyEvalResult {
        let stream = match self.stream_registry.get(stream_id) {
            Some(s) => s,
            None => return PolicyEvalResult::Violation(GovernanceViolation {
                policy_id: uuid::Uuid::nil(),
                policy_name: "ReplayDeterminism".into(),
                severity: PolicySeverity::Error,
                message: format!("Stream '{}' not registered for replay governance", stream_id),
                context: {
                    let mut m = HashMap::new();
                    m.insert("stream_id".into(), stream_id.into());
                    m
                },
                remediations: vec![format!("Register stream '{}' in replay governance", stream_id)],
            }),
        };

        if !stream.deterministic {
            return PolicyEvalResult::Violation(GovernanceViolation {
                policy_id: uuid::Uuid::nil(),
                policy_name: "ReplayDeterminism".into(),
                severity: PolicySeverity::Critical,
                message: format!(
                    "Stream '{}' is not deterministic — replay will produce divergent state",
                    stream_id
                ),
                context: {
                    let mut m = HashMap::new();
                    m.insert("stream_id".into(), stream_id.into());
                    m.insert("deterministic".into(), "false".into());
                    m
                },
                remediations: vec![
                    "Remove non-deterministic operations from event handlers".into(),
                    "Ensure all event processing is idempotent".into(),
                    "Use deterministic data structures in replay".into(),
                ],
            });
        }

        PolicyEvalResult::Pass
    }

    pub fn verify_replay(&mut self, stream_id: &str, event_count: u64, checksum: &[u8]) -> ReplayVerification {
        let mut issues = Vec::new();
        let mut checksum_match = true;
        let mut determinism_verified = true;

        if let Some(stream) = self.stream_registry.get(stream_id) {
            if !stream.checksum.is_empty() && checksum != stream.checksum.as_slice() {
                issues.push(format!(
                    "Checksum mismatch: expected {:x?}, got {:x?}",
                    &stream.checksum[..8],
                    &checksum[..8]
                ));
                checksum_match = false;
            }
        }

        for policy in &self.replay_policies {
            if !checksum_match && policy.require_checksum_verification {
                issues.push("Checksum verification failed — replay may have diverged".into());
                determinism_verified = false;
            }
        }

        let verification = ReplayVerification {
            stream_id: stream_id.to_string(),
            timestamp: chrono::Utc::now(),
            events_replayed: event_count,
            checksum_match,
            determinism_verified,
            divergence_detected: if checksum_match { 0.0 } else { 100.0 },
            issues: issues.clone(),
        };

        self.verification_records.push(verification.clone());
        verification
    }

    pub fn get_safety_report(&self, stream_id: &str) -> ReplaySafetyReport {
        let mut violations = Vec::new();
        let mut determinism_verified = false;
        let mut checksum_valid = false;
        let event_ordering_valid = true;
        let causal_consistency = true;

        if let Some(stream) = self.stream_registry.get(stream_id) {
            determinism_verified = stream.deterministic;
            checksum_valid = !stream.checksum.is_empty();

            if !stream.deterministic {
                violations.push("Stream is not deterministic".into());
            }
            if stream.checksum.is_empty() {
                violations.push("Stream has no checksum for verification".into());
            }
        } else {
            violations.push("Stream not registered in replay governance".into());
        }

        ReplaySafetyReport {
            stream_id: stream_id.to_string(),
            safe: violations.is_empty() && determinism_verified,
            determinism_verified,
            checksum_valid,
            event_ordering_valid,
            causal_consistency,
            violations,
        }
    }

    pub fn get_verification_history(&self) -> &[ReplayVerification] {
        &self.verification_records
    }

    pub fn list_streams(&self) -> Vec<&ReplayStream> {
        self.stream_registry.values().collect()
    }
}

impl Default for ReplayGovernance {
    fn default() -> Self {
        Self::new()
    }
}
