use tracing::info;

/// Threat detection engine — insider threats, replay anomalies, and distributed attack detection
pub struct ThreatEngine;

#[derive(Debug, Clone)]
pub struct ThreatIndicator {
    pub indicator_id: uuid::Uuid,
    pub indicator_type: ThreatIndicatorType,
    pub description: String,
    pub severity: super::SecuritySeverity,
    pub source_node: uuid::Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreatIndicatorType {
    ReplayAttack,
    BruteForceAuth,
    CertificateAnomaly,
    DataExfiltration,
    PrivilegeEscalation,
    FederationAbuse,
    TrustPoisoning,
    GovernanceBypass,
}

#[derive(Debug)]
pub struct InsiderThreatReport {
    pub node_id: uuid::Uuid,
    pub threat_score: f64,
    pub indicators: Vec<ThreatIndicator>,
    pub anomalous_behavior: Vec<String>,
    public_remediation: Vec<String>,
}

#[derive(Debug)]
pub struct ReplayAnomalyReport {
    pub stream_id: String,
    pub expected_checksum: Vec<u8>,
    pub actual_checksum: Vec<u8>,
    pub divergent_events: Vec<u64>,
    pub detected: bool,
}

impl ThreatEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_replay_anomaly(
        &self,
        stream_id: &str,
        expected_checksum: &[u8],
        actual_checksum: &[u8],
        divergent_events: Vec<u64>,
    ) -> ReplayAnomalyReport {
        let detected = expected_checksum != actual_checksum || !divergent_events.is_empty();
        if detected {
            info!(
                stream = %stream_id,
                divergent = divergent_events.len(),
                "Replay anomaly detected"
            );
        }
        ReplayAnomalyReport {
            stream_id: stream_id.to_string(),
            expected_checksum: expected_checksum.to_vec(),
            actual_checksum: actual_checksum.to_vec(),
            divergent_events,
            detected,
        }
    }

    pub fn assess_insider_threat(
        &self,
        node_id: uuid::Uuid,
        indicators: Vec<ThreatIndicator>,
    ) -> InsiderThreatReport {
        let mut behaviors = Vec::new();
        let mut remediation = Vec::new();

        let threat_score: f64 = indicators
            .iter()
            .map(|i| {
                i.confidence
                    * match i.severity {
                        super::SecuritySeverity::Info => 0.1,
                        super::SecuritySeverity::Low => 0.2,
                        super::SecuritySeverity::Medium => 0.4,
                        super::SecuritySeverity::High => 0.7,
                        super::SecuritySeverity::Critical => 1.0,
                    }
            })
            .sum::<f64>()
            / indicators.len().max(1) as f64;

        for indicator in &indicators {
            match indicator.indicator_type {
                ThreatIndicatorType::PrivilegeEscalation => {
                    behaviors.push("Unauthorized privilege escalation attempt".into());
                    remediation.push("Revoke excessive permissions immediately".into());
                }
                ThreatIndicatorType::DataExfiltration => {
                    behaviors.push("Suspicious data access pattern detected".into());
                    remediation
                        .push("Enable data access logging and restrict sensitive data".into());
                }
                ThreatIndicatorType::TrustPoisoning => {
                    behaviors.push("Attempting to manipulate trust scores".into());
                    remediation.push("Audit trust score changes and re-verify all nodes".into());
                }
                ThreatIndicatorType::GovernanceBypass => {
                    behaviors.push("Attempting to bypass governance policies".into());
                    remediation.push("Enforce governance at infrastructure level".into());
                }
                _ => {}
            }
        }

        if threat_score > 0.7 {
            remediation.push("Immediate node isolation and forensic investigation".into());
        }

        InsiderThreatReport {
            node_id,
            threat_score,
            indicators,
            anomalous_behavior: behaviors,
            public_remediation: remediation,
        }
    }
}

impl Default for ThreatEngine {
    fn default() -> Self {
        Self::new()
    }
}
