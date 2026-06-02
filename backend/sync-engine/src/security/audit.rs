use tracing::info;

pub struct SecurityAudit;

impl SecurityAudit {
    pub fn log_sync_auth(
        node_id: &uuid::Uuid,
        peer_id: &uuid::Uuid,
        authenticated: bool,
        reason: Option<&str>,
    ) {
        info!(
            node_id = %node_id,
            peer_id = %peer_id,
            authenticated = authenticated,
            reason = reason,
            "Sync authentication audit"
        );
    }

    pub fn log_sync_signature(
        node_id: &uuid::Uuid,
        sync_id: &uuid::Uuid,
        signature_valid: bool,
    ) {
        info!(
            node_id = %node_id,
            sync_id = %sync_id,
            signature_valid = signature_valid,
            "Sync signature audit"
        );
    }
}
