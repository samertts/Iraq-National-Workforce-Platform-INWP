use tracing::{info, warn};

pub mod threat;
pub mod integrity;

/// Sovereign national security governance — zero-trust enforcement, trust scoring,
/// rogue node detection, insider threat detection, replay anomaly detection
pub struct SecurityGovernanceEngine;

#[derive(Debug, Clone)]
pub struct TrustScore {
    pub node_id: uuid::Uuid,
    pub current_score: f64,
    pub confidence: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub factors: Vec<TrustFactor>,
}

#[derive(Debug, Clone)]
pub struct TrustFactor {
    pub factor_name: String,
    pub weight: f64,
    pub score: f64,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SecurityEvent {
    pub event_id: uuid::Uuid,
    pub event_type: SecurityEventType,
    pub node_id: uuid::Uuid,
    pub severity: SecuritySeverity,
    pub description: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub evidence_hash: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityEventType {
    AuthenticationFailure,
    SignatureVerificationFailed,
    ReplayAnomaly,
    TrustScoreDrop,
    UnauthorizedAccess,
    DataTamperingDetected,
    RogueNodeDetected,
    InsiderThreat,
    FederationViolation,
    CertificateAnomaly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ForensicAuditEntry {
    pub entry_id: uuid::Uuid,
    pub event_id: uuid::Uuid,
    pub node_id: uuid::Uuid,
    pub action: String,
    pub resource: String,
    pub outcome: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub signed_by: Vec<u8>,
    pub previous_hash: Vec<u8>,
    pub entry_hash: Vec<u8>,
}

#[derive(Debug)]
pub struct SecurityPosture {
    pub overall_trust_score: f64,
    pub active_threats: u32,
    pub compromised_nodes: Vec<uuid::Uuid>,
    pub rogue_nodes_detected: u32,
    pub insider_threats: u32,
    pub last_security_audit: chrono::DateTime<chrono::Utc>,
    pub tamper_events: u32,
    pub recommendations: Vec<String>,
}

impl SecurityGovernanceEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_trust_score(&self, factors: Vec<TrustFactor>) -> f64 {
        let total_weight: f64 = factors.iter().map(|f| f.weight).sum();
        if total_weight == 0.0 {
            return 0.5;
        }
        let weighted: f64 = factors.iter().map(|f| f.score * f.weight).sum();
        (weighted / total_weight).clamp(0.0, 1.0)
    }

    pub fn detect_rogue_node(&self, node_id: &uuid::Uuid, events: &[SecurityEvent]) -> Option<RogueNodeReport> {
        let anomaly_count = events.iter()
            .filter(|e| {
                e.node_id == *node_id
                    && matches!(e.event_type, SecurityEventType::AuthenticationFailure
                        | SecurityEventType::SignatureVerificationFailed
                        | SecurityEventType::ReplayAnomaly)
            })
            .count();

        let threat_count = events.iter()
            .filter(|e| {
                e.node_id == *node_id
                    && matches!(e.event_type, SecurityEventType::DataTamperingDetected
                        | SecurityEventType::UnauthorizedAccess)
            })
            .count();

        if anomaly_count > 5 || threat_count > 2 {
            let report = RogueNodeReport {
                node_id: *node_id,
                anomaly_count: anomaly_count as u64,
                threat_count: threat_count as u64,
                confidence: (anomaly_count as f64 / 10.0).min(1.0),
                detected_at: chrono::Utc::now(),
                recommended_action: if threat_count > 2 {
                    "Immediate isolation and forensic investigation".into()
                } else {
                    "Increase monitoring and restrict permissions".into()
                },
            };
            warn!(
                node = %node_id,
                anomalies = anomaly_count,
                threats = threat_count,
                "Rogue node detected"
            );
            return Some(report);
        }

        None
    }

    pub fn generate_security_posture(&self, trust_scores: &[TrustScore], events: &[SecurityEvent]) -> SecurityPosture {
        let avg_trust: f64 = if !trust_scores.is_empty() {
            trust_scores.iter().map(|t| t.current_score).sum::<f64>() / trust_scores.len() as f64
        } else {
            0.0
        };

        let critical_events = events.iter()
            .filter(|e| e.severity >= SecuritySeverity::High)
            .count() as u32;

        let tamper_events = events.iter()
            .filter(|e| matches!(e.event_type, SecurityEventType::DataTamperingDetected))
            .count() as u32;

        let mut recommendations = Vec::new();
        if avg_trust < 0.5 {
            recommendations.push("Critical: overall trust score below 0.5 — initiate federation-wide trust review".into());
        }
        if critical_events > 10 {
            recommendations.push("High volume of critical security events — consider sovereign lockdown".into());
        }
        if tamper_events > 0 {
            recommendations.push("Tampering detected — verify event chain integrity across all domains".into());
        }

        info!(
            trust_score = avg_trust,
            threats = critical_events,
            "Security posture assessment complete"
        );

        SecurityPosture {
            overall_trust_score: avg_trust,
            active_threats: critical_events,
            compromised_nodes: Vec::new(),
            rogue_nodes_detected: 0,
            insider_threats: 0,
            last_security_audit: chrono::Utc::now(),
            tamper_events,
            recommendations,
        }
    }
}

#[derive(Debug)]
pub struct RogueNodeReport {
    pub node_id: uuid::Uuid,
    pub anomaly_count: u64,
    pub threat_count: u64,
    pub confidence: f64,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub recommended_action: String,
}

impl Default for SecurityGovernanceEngine {
    fn default() -> Self {
        Self::new()
    }
}
