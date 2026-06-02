use crate::core::types::SyncPhase;
use crate::error::SyncResult;
use crate::protocol::SyncSession;
use crate::reconciliation::strategy::ConflictStrategy;
use crate::reconciliation::Resolver;
use tracing::{info, warn};

pub struct ReconciliationResult {
    pub resolved_count: u32,
    pub escalated_count: u32,
    pub auto_resolved: u32,
    pub manual_required: u32,
    pub completed: bool,
}

pub async fn perform_reconciliation(
    session: &mut SyncSession,
    partition_key: &str,
    local_records: &[crate::core::types::SyncRecord],
    remote_records: &[crate::core::types::SyncRecord],
    resolver: &Resolver,
) -> SyncResult<ReconciliationResult> {
    info!(
        session_id = %session.session_id,
        partition = %partition_key,
        phase = "reconciliation",
        "Starting reconciliation phase"
    );

    session.advance_phase(SyncPhase::Reconciliation);

    let mut resolved_count = 0_u32;
    let mut escalated_count = 0_u32;
    let mut auto_resolved = 0_u32;
    let mut manual_required = 0_u32;

    let local_by_id: std::collections::HashMap<_, _> = local_records.iter()
        .map(|r| (r.record_id.clone(), r))
        .collect();
    let remote_by_id: std::collections::HashMap<_, _> = remote_records.iter()
        .map(|r| (r.record_id.clone(), r))
        .collect();

    for record_id in local_by_id.keys().chain(remote_by_id.keys()) {
        let local = local_by_id.get(record_id);
        let remote = remote_by_id.get(record_id);

        match (local, remote) {
            (Some(l), Some(r)) => {
                let vv_cmp = l.version_vector.compare(&r.version_vector);
                if vv_cmp == crate::core::version::VersionOrder::Concurrent {
                    let strategy = resolver.matrix
                        .get_strategy(&l.record_type)
                        .unwrap_or(&ConflictStrategy::Lww);

                    match strategy {
                        ConflictStrategy::Lww => {
                            auto_resolved += 1;
                            resolved_count += 1;
                        }
                        ConflictStrategy::MinistryAuthor => {
                            auto_resolved += 1;
                            resolved_count += 1;
                        }
                        ConflictStrategy::Manual => {
                            manual_required += 1;
                            escalated_count += 1;
                            warn!(
                                record_id = %record_id,
                                record_type = %l.record_type,
                                "Conflict requires manual resolution"
                            );
                        }
                        _ => {
                            auto_resolved += 1;
                            resolved_count += 1;
                        }
                    }
                }
            }
            _ => {
                resolved_count += 1;
            }
        }
    }

    info!(
        partition = %partition_key,
        resolved = resolved_count,
        escalated = escalated_count,
        auto = auto_resolved,
        manual = manual_required,
        "Reconciliation completed"
    );

    Ok(ReconciliationResult {
        resolved_count,
        escalated_count,
        auto_resolved,
        manual_required,
        completed: manual_required == 0,
    })
}
