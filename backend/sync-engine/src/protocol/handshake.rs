use crate::error::SyncResult;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct HandshakeResult {
    pub accepted: bool,
    pub schema_version: String,
    pub compatible: bool,
    pub reject_reason: Option<String>,
    pub suggested_compression: String,
}

pub async fn perform_handshake(
    local_versions: &[String],
    remote_versions: &[String],
) -> SyncResult<HandshakeResult> {
    info!("Performing schema handshake");

    let compatible = local_versions.iter().any(|v| remote_versions.contains(v));

    if !compatible {
        warn!(
            local = ?local_versions,
            remote = ?remote_versions,
            "Schema version mismatch"
        );
        return Ok(HandshakeResult {
            accepted: false,
            schema_version: String::new(),
            compatible: false,
            reject_reason: Some(format!(
                "No compatible schema version. Local: {:?}, Remote: {:?}",
                local_versions, remote_versions
            )),
            suggested_compression: String::new(),
        });
    }

    let negotiated_version = local_versions
        .iter()
        .find(|v| remote_versions.contains(v))
        .cloned()
        .unwrap_or_default();

    info!(
        version = %negotiated_version,
        "Schema handshake successful"
    );

    Ok(HandshakeResult {
        accepted: true,
        schema_version: negotiated_version,
        compatible: true,
        reject_reason: None,
        suggested_compression: "zstd".into(),
    })
}
