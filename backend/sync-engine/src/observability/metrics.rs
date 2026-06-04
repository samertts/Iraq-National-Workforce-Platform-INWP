use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};

pub fn init_sync_metrics() {
    describe_counter!(
        "sync.batches.total",
        "Total number of sync batches processed"
    );
    describe_counter!("sync.events.synced", "Total number of events synced");
    describe_counter!("sync.conflicts.detected", "Total conflicts detected");
    describe_counter!("sync.conflicts.resolved", "Total conflicts resolved");
    describe_counter!("sync.recovery.attempts", "Total recovery attempts");
    describe_gauge!("sync.queue.depth", "Current sync queue depth");
    describe_gauge!("sync.peers.connected", "Number of connected peers");
    describe_gauge!("sync.pending.conflicts", "Number of pending conflicts");
    describe_histogram!("sync.batch.duration_ms", "Duration of sync batches in ms");
    describe_histogram!("sync.delta.size_bytes", "Size of delta transfers in bytes");
}

pub fn record_batch_completed(duration_ms: f64) {
    counter!("sync.batches.total").increment(1);
    histogram!("sync.batch.duration_ms").record(duration_ms);
}

pub fn record_events_synced(count: u64) {
    counter!("sync.events.synced").increment(count);
}

pub fn record_conflict_detected() {
    counter!("sync.conflicts.detected").increment(1);
}

pub fn record_conflict_resolved() {
    counter!("sync.conflicts.resolved").increment(1);
}

pub fn record_recovery_attempt() {
    counter!("sync.recovery.attempts").increment(1);
}

pub fn set_queue_depth(depth: u64) {
    gauge!("sync.queue.depth").set(depth as f64);
}

pub fn set_peers_connected(count: u64) {
    gauge!("sync.peers.connected").set(count as f64);
}

pub fn set_pending_conflicts(count: u64) {
    gauge!("sync.pending.conflicts").set(count as f64);
}

pub fn record_delta_size(size_bytes: f64) {
    histogram!("sync.delta.size_bytes").record(size_bytes);
}
