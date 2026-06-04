use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStrategy {
    Lww,
    MinistryAuthor,
    ServiceMerge,
    AdditiveMerge,
    Manual,
}

impl ConflictStrategy {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lww => "lww",
            Self::MinistryAuthor => "ministry_author",
            Self::ServiceMerge => "service_merge",
            Self::AdditiveMerge => "additive_merge",
            Self::Manual => "manual",
        }
    }

    pub fn is_auto_resolvable(&self) -> bool {
        matches!(self, Self::Lww | Self::MinistryAuthor | Self::AdditiveMerge)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionMatrix {
    strategies: HashMap<String, ConflictStrategy>,
    default_strategy: ConflictStrategy,
}

impl ResolutionMatrix {
    pub fn new(default_strategy: ConflictStrategy) -> Self {
        Self {
            strategies: HashMap::new(),
            default_strategy,
        }
    }

    pub fn register(&mut self, entity_type: impl Into<String>, strategy: ConflictStrategy) {
        self.strategies.insert(entity_type.into(), strategy);
    }

    pub fn get_strategy(&self, entity_type: &str) -> Option<&ConflictStrategy> {
        self.strategies
            .get(entity_type)
            .or(Some(&self.default_strategy))
    }

    pub fn is_auto_resolvable(&self, entity_type: &str) -> bool {
        self.get_strategy(entity_type)
            .map(|s| s.is_auto_resolvable())
            .unwrap_or(false)
    }
}

impl Default for ResolutionMatrix {
    fn default() -> Self {
        let mut matrix = Self::new(ConflictStrategy::Lww);

        matrix.register("ClockEvent", ConflictStrategy::Lww);
        matrix.register("AttendanceException", ConflictStrategy::Lww);
        matrix.register("Shift", ConflictStrategy::MinistryAuthor);
        matrix.register("AttendancePolicy", ConflictStrategy::MinistryAuthor);
        matrix.register("LeaveRequest", ConflictStrategy::ServiceMerge);
        matrix.register("LeaveBalance", ConflictStrategy::ServiceMerge);
        matrix.register("AccrualPolicy", ConflictStrategy::Lww);
        matrix.register("UserProfile", ConflictStrategy::Lww);
        matrix.register("RoleAssignment", ConflictStrategy::AdditiveMerge);
        matrix.register("DeviceTrust", ConflictStrategy::Lww);
        matrix.register("LedgerEntry", ConflictStrategy::Manual);
        matrix.register("PolicyDefinition", ConflictStrategy::MinistryAuthor);
        matrix.register("SyncConfig", ConflictStrategy::Lww);
        matrix.register("NodeRegistry", ConflictStrategy::MinistryAuthor);
        matrix.register("SyncCheckpoint", ConflictStrategy::Lww);

        matrix
    }
}
