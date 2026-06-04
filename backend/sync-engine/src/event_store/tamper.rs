use super::chain::EventChain;
use sha2::{Digest, Sha256};
use tracing::warn;

pub struct TamperDetector {
    expected_genesis: Vec<u8>,
}

impl Default for TamperDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl TamperDetector {
    pub fn new() -> Self {
        Self {
            expected_genesis: Sha256::new()
                .chain_update(b"INWP_GENESIS")
                .finalize()
                .to_vec(),
        }
    }

    pub fn detect_tampering(&self, chain: &EventChain) -> TamperReport {
        let mut issues = Vec::new();

        // 1. Genesis check
        if chain.chain.is_empty() {
            return TamperReport {
                tampered: false,
                issues: Vec::new(),
                tampered_events: 0,
            };
        }

        if chain.chain[0].previous_hash != self.expected_genesis {
            issues.push(TamperIssue {
                severity: TamperSeverity::Critical,
                description: "Genesis hash mismatch — chain may have been replaced".into(),
                affected_depth: 0,
            });
        }

        // 2. Sequential hash chain verification
        for window in chain.chain.windows(2) {
            if window[1].previous_hash != window[0].event_hash {
                issues.push(TamperIssue {
                    severity: TamperSeverity::Critical,
                    description: format!(
                        "Hash chain broken between depth {} and {}",
                        window[0].chain_depth, window[1].chain_depth
                    ),
                    affected_depth: window[1].chain_depth,
                });
            }
        }

        // 3. Individual event integrity
        for event in &chain.chain {
            if !event.verify_chain_integrity() {
                issues.push(TamperIssue {
                    severity: TamperSeverity::High,
                    description: format!(
                        "Event integrity check failed at depth {}",
                        event.chain_depth
                    ),
                    affected_depth: event.chain_depth,
                });
            }
        }

        // 4. Depth continuity
        for (i, event) in chain.chain.iter().enumerate() {
            if event.chain_depth != i as u64 {
                issues.push(TamperIssue {
                    severity: TamperSeverity::Major,
                    description: format!(
                        "Depth discontinuity: expected {}, got {}",
                        i, event.chain_depth
                    ),
                    affected_depth: event.chain_depth,
                });
            }
        }

        if !issues.is_empty() {
            warn!(
                issue_count = issues.len(),
                "Tampering detected in event chain"
            );
        }

        let tampered = !issues.is_empty();
        let tampered_events = issues.len() as u64;
        TamperReport {
            tampered,
            issues,
            tampered_events,
        }
    }

    pub fn verify_seal(&self, chain: &EventChain, expected_seal: &[u8]) -> bool {
        if chain.chain.is_empty() {
            return expected_seal.is_empty();
        }

        let mut hasher = Sha256::new();
        for event in &chain.chain {
            hasher.update(&event.event_hash);
        }
        let computed_seal = hasher.finalize().to_vec();
        computed_seal == expected_seal
    }
}

#[derive(Debug)]
pub struct TamperReport {
    pub tampered: bool,
    pub issues: Vec<TamperIssue>,
    pub tampered_events: u64,
}

#[derive(Debug)]
pub struct TamperIssue {
    pub severity: TamperSeverity,
    pub description: String,
    pub affected_depth: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TamperSeverity {
    Low,
    Major,
    High,
    Critical,
}
