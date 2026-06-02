pub mod mesh;
pub mod routing;
pub mod topology;

pub use mesh::*;
pub use routing::*;
pub use topology::*;

use serde::{Deserialize, Serialize};

/// Sovereign federation tier — each level represents a domain of authority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FederationTier {
    NationalSovereign,
    MinistryDomain,
    RegionalGovernorate,
    InstitutionZone,
    EdgeSite,
}

impl FederationTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NationalSovereign => "sovereign",
            Self::MinistryDomain => "ministry",
            Self::RegionalGovernorate => "regional",
            Self::InstitutionZone => "institution",
            Self::EdgeSite => "edge",
        }
    }

    pub fn authority_rank(&self) -> u8 {
        match self {
            Self::NationalSovereign => 5,
            Self::MinistryDomain => 4,
            Self::RegionalGovernorate => 3,
            Self::InstitutionZone => 2,
            Self::EdgeSite => 1,
        }
    }

    pub fn can_override(&self, other: &FederationTier) -> bool {
        self.authority_rank() >= other.authority_rank()
    }
}

/// Federated domain identity — globally unique within the sovereign system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationDomain {
    pub domain_id: uuid::Uuid,
    pub tier: FederationTier,
    pub parent_domain: Option<uuid::Uuid>,
    pub authority_key: Vec<u8>,
    pub sovereignty_boundary: SovereigntyBoundary,
    pub name_arabic: String,
    pub name_english: String,
    pub jurisdiction: String,
    pub schema_versions: Vec<String>,
}

/// Defines what crosses a sovereignty boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyBoundary {
    pub domain_id: uuid::Uuid,
    pub allow_inbound_schemas: Vec<String>,
    pub allow_outbound_schemas: Vec<String>,
    pub require_approval_for: Vec<String>,
    pub max_replication_lag_seconds: u64,
    pub conflict_resolution_authority: ConflictAuthority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictAuthority {
    LocalSovereign,
    ParentDomain,
    Negotiated,
    EscalateNational,
}

/// Federation routing table — determines where events flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationRoute {
    pub source_domain: uuid::Uuid,
    pub target_domain: uuid::Uuid,
    pub route_type: RouteType,
    pub sync_schedule: SyncSchedule,
    pub compression_policy: CompressionPolicy,
    pub bandwidth_budget_bytes_per_sec: u64,
    pub encrypted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteType {
    ParentChild,
    Peer,
    DisasterRecovery,
    AuditOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSchedule {
    pub continuous: bool,
    pub batch_intervals_secs: Vec<u64>,
    pub priority_window: Option<PriorityWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityWindow {
    pub start_utc_hour: u8,
    pub end_utc_hour: u8,
    pub max_bandwidth_percent: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionPolicy {
    Maximum,   // zstd L19
    Balanced,  // zstd L3
    Minimum,   // no compression
    Adaptive,  // based on bandwidth
}
