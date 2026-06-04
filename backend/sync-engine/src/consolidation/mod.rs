pub mod automation;
pub mod control;
pub mod evolution;
pub mod freeze;
pub mod graph;
pub mod healing;
pub mod observer;
pub mod operations;
pub mod security;

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
    pub fn verify_invariants(
        &self,
        registry: &freeze::InvariantRegistry,
    ) -> freeze::InvariantComplianceReport {
        registry.verify_all()
    }
}
