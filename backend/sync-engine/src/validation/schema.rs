use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersion {
    pub version: String,
    pub supported_entities: Vec<String>,
    pub event_types: Vec<String>,
    pub compression_algorithms: Vec<String>,
}

impl SchemaVersion {
    pub fn current() -> Self {
        Self {
            version: "1.0".into(),
            supported_entities: vec![
                "ClockEvent".into(),
                "AttendanceException".into(),
                "Shift".into(),
                "AttendancePolicy".into(),
                "LeaveRequest".into(),
                "LeaveBalance".into(),
                "UserProfile".into(),
                "RoleAssignment".into(),
                "DeviceTrust".into(),
                "LedgerEntry".into(),
                "PolicyDefinition".into(),
            ],
            event_types: vec![
                "inwp.sync.v1.batch.committed".into(),
                "inwp.sync.v1.conflict.detected".into(),
                "inwp.sync.v1.conflict.resolved".into(),
                "inwp.sync.v1.heartbeat.sent".into(),
                "inwp.sync.v1.node.state.changed".into(),
            ],
            compression_algorithms: vec!["zstd".into(), "none".into()],
        }
    }

    pub fn is_compatible(&self, other: &SchemaVersion) -> bool {
        self.supported_entities.iter().any(|e| other.supported_entities.contains(e))
            && self.compression_algorithms.iter().any(|c| other.compression_algorithms.contains(c))
    }
}

pub async fn negotiate_schema(
    local: &SchemaVersion,
    remote: &SchemaVersion,
) -> SchemaNegotiationResult {
    if local.version == remote.version {
        return SchemaNegotiationResult {
            agreed_version: local.version.clone(),
            compatible: true,
        };
    }

    let compatible = local.is_compatible(remote);

    SchemaNegotiationResult {
        agreed_version: if compatible {
            // Use the lower version
            std::cmp::min(&local.version, &remote.version).clone()
        } else {
            String::new()
        },
        compatible,
    }
}

pub struct SchemaNegotiationResult {
    pub agreed_version: String,
    pub compatible: bool,
}
