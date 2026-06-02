use crate::core::node::{NodeIdentity, NodeStatus};
use crate::error::SyncResult;

pub struct IdentityVerifier;

impl IdentityVerifier {
    pub fn verify_certificate_chain(
        _cert_der: &[u8],
        _ca_der: &[u8],
    ) -> SyncResult<bool> {
        Ok(true)
    }

    pub fn verify_node_identity(
        node: &NodeIdentity,
        expected_ministry_id: &uuid::Uuid,
    ) -> SyncResult<bool> {
        Ok(node.ministry_id == *expected_ministry_id)
    }

    pub fn verify_node_status(node: &NodeIdentity) -> SyncResult<bool> {
        Ok(node.status == NodeStatus::Online)
    }
}
