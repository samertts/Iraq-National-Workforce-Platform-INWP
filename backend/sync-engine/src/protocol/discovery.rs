use crate::core::node::{NodeIdentity, NodeType};
use crate::core::partition::PartitionKey;
use crate::core::types::SyncPhase;
use crate::error::SyncResult;
use crate::protocol::SyncSession;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    pub peer: NodeIdentity,
    pub partitions: Vec<String>,
    pub accepted: bool,
    pub reject_reason: Option<String>,
    pub suggested_batch_size: u32,
}

pub async fn perform_discovery(
    session: &mut SyncSession,
    local_node: &NodeIdentity,
    remote_node: &NodeIdentity,
) -> SyncResult<DiscoveryResult> {
    info!(
        session_id = %session.session_id,
        peer = %remote_node.node_id,
        peer_type = %remote_node.node_type,
        phase = "discovery",
        "Starting discovery phase"
    );

    session.advance_phase(SyncPhase::Discovery);

    if !local_node.can_initiate_sync(remote_node) {
        warn!(
            session_id = %session.session_id,
            reason = "Node not authorized to initiate sync with this peer type",
            "Discovery rejected"
        );
        return Ok(DiscoveryResult {
            peer: remote_node.clone(),
            partitions: Vec::new(),
            accepted: false,
            reject_reason: Some(
                "Sync topology violation: node cannot initiate sync with this peer type".into(),
            ),
            suggested_batch_size: 0,
        });
    }

    let common_partitions = compute_common_partitions(local_node, remote_node);

    if common_partitions.is_empty() {
        warn!(
            session_id = %session.session_id,
            "No common partitions found"
        );
        return Ok(DiscoveryResult {
            peer: remote_node.clone(),
            partitions: Vec::new(),
            accepted: false,
            reject_reason: Some("No common partitions to sync".into()),
            suggested_batch_size: 0,
        });
    }

    info!(
        session_id = %session.session_id,
        partition_count = common_partitions.len(),
        "Discovery completed successfully"
    );

    Ok(DiscoveryResult {
        peer: remote_node.clone(),
        partitions: common_partitions,
        accepted: true,
        reject_reason: None,
        suggested_batch_size: 1000,
    })
}

fn compute_common_partitions(local: &NodeIdentity, remote: &NodeIdentity) -> Vec<String> {
    let mut partitions = Vec::new();

    if local.ministry_id == remote.ministry_id {
        partitions.push(PartitionKey::ministry_prefix(local.ministry_id));

        match (local.node_type, remote.node_type) {
            (NodeType::Edge, NodeType::Edge) => {
                if local.site_id == remote.site_id {
                    partitions.push("local/site".into());
                }
            }
            (NodeType::Edge, NodeType::RegionalRelay) => {
                partitions.push(PartitionKey::entity_prefix(
                    local.ministry_id,
                    "clock_events",
                ));
                partitions.push(PartitionKey::entity_prefix(local.ministry_id, "attendance"));
            }
            (NodeType::RegionalRelay, NodeType::NationalHub) => {
                partitions.push(PartitionKey::entity_prefix(local.ministry_id, "sync_meta"));
            }
            _ => {}
        }
    }

    partitions
}
