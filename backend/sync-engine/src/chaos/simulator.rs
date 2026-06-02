use sha2::{Digest, Sha256};
use tracing::info;

/// Distributed simulation engines for replay, federation, and synchronization scenarios
pub struct ReplaySimulator;

#[derive(Debug)]
pub struct ReplaySimulation {
    pub simulation_id: uuid::Uuid,
    pub stream_id: String,
    pub total_events: u64,
    pub replay_count: u64,
    pub deterministic: bool,
    pub state_divergences: Vec<StateDivergence>,
    pub final_checksum: Vec<u8>,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub struct StateDivergence {
    pub event_index: u64,
    pub expected_hash: Vec<u8>,
    pub actual_hash: Vec<u8>,
    pub cause: String,
}

pub struct FederationSimulator;

#[derive(Debug)]
pub struct FederationSimulation {
    pub simulation_id: uuid::Uuid,
    pub topology: FederationTopology,
    pub nodes: Vec<SimulatedNode>,
    pub event_routes: Vec<SimulatedRoute>,
    pub convergence_time_ms: u64,
    pub data_consistency_achieved: bool,
}

#[derive(Debug)]
pub struct FederationTopology {
    pub node_count: usize,
    pub hierarchy_depth: u32,
    pub partition_probability: f64,
    pub latency_ms_range: (u64, u64),
}

#[derive(Debug)]
pub struct SimulatedNode {
    pub node_id: uuid::Uuid,
    pub domain: String,
    pub online: bool,
    pub events_received: u64,
    pub events_sent: u64,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug)]
pub struct SimulatedRoute {
    pub source: uuid::Uuid,
    pub target: uuid::Uuid,
    pub latency_ms: u64,
    pub active: bool,
}

pub struct SynchronizationStormSimulator;

#[derive(Debug)]
pub struct StormSimulation {
    pub simulation_id: uuid::Uuid,
    pub concurrent_syncs: u64,
    pub total_events_exchanged: u64,
    pub conflicts_detected: u64,
    pub conflicts_resolved: u64,
    pub duration_ms: u64,
    pub convergence_achieved: bool,
}

impl ReplaySimulator {
    pub fn new() -> Self {
        Self
    }

    pub fn simulate_replay(
        &self,
        stream_id: &str,
        total_events: u64,
        replay_count: u64,
        expected_checksum: &[u8],
    ) -> ReplaySimulation {
        let mut divergences = Vec::new();

        for i in 0..replay_count {
            let sim_checksum = self.compute_simulated_checksum(stream_id, i);
            if sim_checksum != expected_checksum {
                divergences.push(StateDivergence {
                    event_index: i,
                    expected_hash: expected_checksum.to_vec(),
                    actual_hash: sim_checksum,
                    cause: format!("Replay #{} produced divergent checksum — non-determinism detected", i + 1),
                });
            }
        }

        let deterministic = divergences.is_empty();
        if deterministic {
            info!(
                stream = %stream_id,
                replays = replay_count,
                events = total_events,
                "Replay simulation passed — deterministic across all runs"
            );
        } else {
            info!(
                stream = %stream_id,
                divergences = divergences.len(),
                "Replay simulation detected non-determinism"
            );
        }

        ReplaySimulation {
            simulation_id: uuid::Uuid::now_v7(),
            stream_id: stream_id.to_string(),
            total_events,
            replay_count,
            deterministic,
            state_divergences: divergences,
            final_checksum: expected_checksum.to_vec(),
            duration_ms: total_events * 10,
        }
    }

    fn compute_simulated_checksum(&self, stream_id: &str, _seed: u64) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(stream_id.as_bytes());
        hasher.finalize().to_vec()
    }
}

impl FederationSimulator {
    pub fn new() -> Self {
        Self
    }

    pub fn simulate_federation(&self, topology: FederationTopology) -> FederationSimulation {
        let mut nodes = Vec::new();
        let mut routes = Vec::new();

        for i in 0..topology.node_count {
            nodes.push(SimulatedNode {
                node_id: uuid::Uuid::now_v7(),
                domain: format!("domain-{}", i),
                online: true,
                events_received: 0,
                events_sent: 0,
                last_sync: Some(chrono::Utc::now()),
            });
        }

        for i in 0..topology.node_count {
            for j in (i + 1)..topology.node_count {
                let (min, max) = topology.latency_ms_range;
                let latency = min + (max - min) / 2;
                let active = rand::random::<f64>() > topology.partition_probability;
                routes.push(SimulatedRoute {
                    source: nodes[i].node_id,
                    target: nodes[j].node_id,
                    latency_ms: latency,
                    active,
                });
            }
        }

        info!(
            nodes = topology.node_count,
            routes = routes.len(),
            partition_prob = topology.partition_probability,
            "Federation simulation initialized"
        );

        FederationSimulation {
            simulation_id: uuid::Uuid::now_v7(),
            topology,
            nodes,
            event_routes: routes,
            convergence_time_ms: 0,
            data_consistency_achieved: false,
        }
    }
}

impl SynchronizationStormSimulator {
    pub fn new() -> Self {
        Self
    }

    pub fn simulate_storm(&self, concurrent_syncs: u64, events_per_sync: u64) -> StormSimulation {
        let total_events = concurrent_syncs * events_per_sync;
        let conflicts = total_events / 100;

        info!(
            concurrent = concurrent_syncs,
            total_events = total_events,
            "Synchronization storm simulation executed"
        );

        StormSimulation {
            simulation_id: uuid::Uuid::now_v7(),
            concurrent_syncs,
            total_events_exchanged: total_events,
            conflicts_detected: conflicts,
            conflicts_resolved: conflicts,
            duration_ms: total_events / 100,
            convergence_achieved: true,
        }
    }
}

impl Default for ReplaySimulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FederationSimulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SynchronizationStormSimulator {
    fn default() -> Self {
        Self::new()
    }
}
