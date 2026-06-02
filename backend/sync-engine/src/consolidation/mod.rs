pub mod freeze;
pub mod observer;
pub mod healing;
pub mod graph;
pub mod control;
pub mod operations;
pub mod automation;
pub mod security;
pub mod evolution;

/// Self-governing sovereign infrastructure consolidation.
/// All invariants are frozen, all evolution is deterministic, all operations are replay-safe.
pub struct ConsolidationEngine;

impl Default for ConsolidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsolidationEngine {
    pub fn new() -> Self {
        Self
    }

    /// Verify all frozen invariants are satisfied across the platform
    pub fn verify_invariants(&self, registry: &freeze::InvariantRegistry) -> freeze::InvariantComplianceReport {
        registry.verify_all()
    }
}
