use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    NationalHub,
    RegionalRelay,
    Edge,
    Mobile,
    DrReplica,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NationalHub => "national_hub",
            Self::RegionalRelay => "regional_relay",
            Self::Edge => "edge",
            Self::Mobile => "mobile",
            Self::DrReplica => "dr_replica",
        }
    }

    pub fn tier_rank(&self) -> u8 {
        match self {
            Self::NationalHub => 5,
            Self::RegionalRelay => 4,
            Self::DrReplica => 3,
            Self::Edge => 2,
            Self::Mobile => 1,
        }
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&crate::config::NodeTypeConfig> for NodeType {
    fn from(cfg: &crate::config::NodeTypeConfig) -> Self {
        match cfg {
            crate::config::NodeTypeConfig::NationalHub => Self::NationalHub,
            crate::config::NodeTypeConfig::RegionalRelay => Self::RegionalRelay,
            crate::config::NodeTypeConfig::Edge => Self::Edge,
            crate::config::NodeTypeConfig::Mobile => Self::Mobile,
            crate::config::NodeTypeConfig::DrReplica => Self::DrReplica,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Offline,
    Suspected,
    Recovering,
    Quarantined,
}

impl NodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
            Self::Suspected => "suspected",
            Self::Recovering => "recovering",
            Self::Quarantined => "quarantined",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub schema_versions: Vec<String>,
    pub supported_entities: Vec<String>,
    pub max_partitions: u32,
    pub max_batch_size_bytes: u64,
    pub supports_compression: bool,
    pub compression_algorithms: Vec<String>,
    pub supports_lan_mesh: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub node_id: uuid::Uuid,
    pub node_type: NodeType,
    pub node_name: String,
    pub ministry_id: uuid::Uuid,
    pub site_id: uuid::Uuid,
    pub region: String,
    pub certificate_serial: String,
    pub public_key: Vec<u8>,
    pub address: String,
    pub port: u16,
    pub capabilities: Capabilities,
    pub status: NodeStatus,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

impl NodeIdentity {
    pub fn is_peer(&self, other: &NodeIdentity) -> bool {
        self.region == other.region
            && self.ministry_id == other.ministry_id
            && self.node_id != other.node_id
    }

    pub fn can_initiate_sync(&self, target: &NodeIdentity) -> bool {
        match (self.node_type, target.node_type) {
            (NodeType::Edge, NodeType::Edge) => true,
            (NodeType::Edge, NodeType::RegionalRelay) => true,
            (NodeType::Edge, NodeType::NationalHub) => false,
            (NodeType::RegionalRelay, NodeType::RegionalRelay) => true,
            (NodeType::RegionalRelay, NodeType::NationalHub) => true,
            (NodeType::NationalHub, NodeType::RegionalRelay) => true,
            _ => self.node_type.tier_rank() <= target.node_type.tier_rank(),
        }
    }
}
