use crate::core::crdt::{GSet, PnCounter};
use crate::core::node::NodeIdentity;
use crate::core::version::VersionVector;
use crate::error::SyncResult;
use crate::reconciliation::strategy::{ConflictStrategy, ResolutionMatrix};
use serde::{Deserialize, Serialize};

pub struct Resolver {
    pub matrix: ResolutionMatrix,
}

impl Resolver {
    pub fn new(matrix: ResolutionMatrix) -> Self {
        Self { matrix }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn resolve(
        &self,
        record_type: &str,
        local_payload: &[u8],
        remote_payload: &[u8],
        local_vv: &VersionVector,
        remote_vv: &VersionVector,
        local_node: &NodeIdentity,
        remote_node: &NodeIdentity,
    ) -> SyncResult<Resolution> {
        let strategy = self
            .matrix
            .get_strategy(record_type)
            .cloned()
            .unwrap_or(ConflictStrategy::Lww);

        match strategy {
            ConflictStrategy::Lww => {
                self.resolve_lww(local_payload, remote_payload, local_vv, remote_vv)
            }
            ConflictStrategy::MinistryAuthor => {
                self.resolve_ministry_author(local_payload, remote_payload, local_node, remote_node)
            }
            ConflictStrategy::ServiceMerge => {
                self.resolve_service_merge(record_type, local_payload, remote_payload)
            }
            ConflictStrategy::AdditiveMerge => {
                self.resolve_additive_merge(local_payload, remote_payload)
            }
            ConflictStrategy::Manual => Ok(Resolution::Escalated(ConflictEscalation {
                record_type: record_type.to_string(),
                reason: "Manual resolution required by strategy".into(),
                local_payload: local_payload.to_vec(),
                remote_payload: remote_payload.to_vec(),
                local_version: local_vv.clone(),
                remote_version: remote_vv.clone(),
            })),
        }
    }

    fn resolve_lww(
        &self,
        _local: &[u8],
        _remote: &[u8],
        local_vv: &VersionVector,
        remote_vv: &VersionVector,
    ) -> SyncResult<Resolution> {
        if local_vv.local_timestamp >= remote_vv.local_timestamp {
            Ok(Resolution::AcceptLocal)
        } else {
            Ok(Resolution::AcceptRemote)
        }
    }

    fn resolve_ministry_author(
        &self,
        _local: &[u8],
        _remote: &[u8],
        local_node: &NodeIdentity,
        remote_node: &NodeIdentity,
    ) -> SyncResult<Resolution> {
        if remote_node.node_type.tier_rank() >= local_node.node_type.tier_rank() {
            Ok(Resolution::AcceptRemote)
        } else {
            Ok(Resolution::AcceptLocal)
        }
    }

    fn resolve_service_merge(
        &self,
        record_type: &str,
        local: &[u8],
        remote: &[u8],
    ) -> SyncResult<Resolution> {
        match record_type {
            "LeaveBalance" => {
                let local_balance: i64 = serde_json::from_slice(local).unwrap_or(0);
                let remote_balance: i64 = serde_json::from_slice(remote).unwrap_or(0);

                let mut counter = PnCounter::new();
                if local_balance >= 0 {
                    counter.increment(uuid::Uuid::nil(), local_balance as u64);
                } else {
                    counter.decrement(uuid::Uuid::nil(), (-local_balance) as u64);
                }
                if remote_balance >= 0 {
                    counter.increment(uuid::Uuid::nil(), remote_balance as u64);
                } else {
                    counter.decrement(uuid::Uuid::nil(), (-remote_balance) as u64);
                }

                let merged_value = counter.value();
                Ok(Resolution::Merged(serde_json::to_vec(&merged_value)?))
            }
            _ => {
                let mut local_set = GSet::new();
                if let Ok(elements) = serde_json::from_slice::<Vec<String>>(local) {
                    for e in elements {
                        local_set.add(e.into_bytes());
                    }
                }
                let mut remote_set = GSet::new();
                if let Ok(elements) = serde_json::from_slice::<Vec<String>>(remote) {
                    for e in elements {
                        remote_set.add(e.into_bytes());
                    }
                }
                let merged = local_set.merge(&remote_set);
                let merged_strs: Vec<String> = merged
                    .elements
                    .iter()
                    .filter_map(|e| String::from_utf8(e.clone()).ok())
                    .collect();
                Ok(Resolution::Merged(serde_json::to_vec(&merged_strs)?))
            }
        }
    }

    fn resolve_additive_merge(&self, local: &[u8], remote: &[u8]) -> SyncResult<Resolution> {
        let local_set: GSet = serde_json::from_slice(local).unwrap_or_else(|_| {
            let mut s = GSet::new();
            s.add(local.to_vec());
            s
        });
        let remote_set: GSet = serde_json::from_slice(remote).unwrap_or_else(|_| {
            let mut s = GSet::new();
            s.add(remote.to_vec());
            s
        });

        let merged = local_set.merge(&remote_set);
        Ok(Resolution::Merged(serde_json::to_vec(&merged)?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Resolution {
    AcceptLocal,
    AcceptRemote,
    Merged(Vec<u8>),
    Escalated(ConflictEscalation),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictEscalation {
    pub record_type: String,
    pub reason: String,
    pub local_payload: Vec<u8>,
    pub remote_payload: Vec<u8>,
    pub local_version: VersionVector,
    pub remote_version: VersionVector,
}
