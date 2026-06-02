pub mod simulator;
pub mod partition;
pub mod corruption;
pub mod byzantine;
pub mod recovery_test;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Sovereign distributed chaos engineering infrastructure.
/// Tests the platform under partition, corruption, Byzantine behavior, and catastrophic failure.
pub struct ChaosEngine {
    experiments: Arc<RwLock<Vec<ChaosExperiment>>>,
    results: Arc<RwLock<Vec<ExperimentResult>>>,
    active_simulations: Arc<RwLock<Vec<SimulationInstance>>>,
}

#[derive(Debug, Clone)]
pub struct ChaosExperiment {
    pub experiment_id: uuid::Uuid,
    pub name: String,
    pub experiment_type: ExperimentType,
    pub target_domain: String,
    pub parameters: HashMap<String, String>,
    pub duration_secs: u64,
    pub injection_mode: InjectionMode,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExperimentType {
    Partition,
    Corruption,
    Byzantine,
    Latency,
    ReplayDivergence,
    FederationStorm,
    EventStorm,
    RecoveryStress,
    EdgeIsolation,
    SplitBrain,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InjectionMode {
    Once,
    Periodic(u64),
    Random(f64),
    Continuous,
}

#[derive(Debug, Clone)]
pub struct SimulationInstance {
    pub simulation_id: uuid::Uuid,
    pub experiment_id: uuid::Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub status: SimulationStatus,
    pub target_components: Vec<String>,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationStatus {
    Initializing,
    Running,
    Injecting,
    Observing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct ExperimentResult {
    pub experiment_id: uuid::Uuid,
    pub completed_at: chrono::DateTime<chrono::Utc>,
    pub passed: bool,
    pub duration_secs: u64,
    pub observations: Vec<String>,
    pub failures: Vec<String>,
    pub metrics: HashMap<String, f64>,
    pub system_survived: bool,
    pub data_integrity_verified: bool,
    pub replay_safety_verified: bool,
}

impl ChaosEngine {
    pub fn new() -> Self {
        Self {
            experiments: Arc::new(RwLock::new(Vec::new())),
            results: Arc::new(RwLock::new(Vec::new())),
            active_simulations: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register_experiment(&self, experiment: ChaosExperiment) {
        info!(
            name = %experiment.name,
            kind = ?experiment.experiment_type,
            target = %experiment.target_domain,
            "Chaos experiment registered"
        );
        self.experiments.write().await.push(experiment);
    }

    pub async fn start_experiment(&self, experiment_id: uuid::Uuid) -> Option<SimulationInstance> {
        let experiment = self.experiments.read().await.iter().find(|e| e.experiment_id == experiment_id)?.clone();

        let simulation = SimulationInstance {
            simulation_id: uuid::Uuid::now_v7(),
            experiment_id,
            started_at: chrono::Utc::now(),
            status: SimulationStatus::Running,
            target_components: vec![experiment.target_domain],
            metrics: HashMap::new(),
        };

        info!(
            simulation = %simulation.simulation_id,
            experiment = %experiment_id,
            "Chaos simulation started"
        );

        self.active_simulations.write().await.push(simulation.clone());
        Some(simulation)
    }

    pub async fn inject_fault(
        &self,
        simulation_id: uuid::Uuid,
        fault_type: ExperimentType,
    ) {
        let mut sims = self.active_simulations.write().await;
        if let Some(sim) = sims.iter_mut().find(|s| s.simulation_id == simulation_id) {
            sim.status = SimulationStatus::Injecting;
            let mut metrics = HashMap::new();
            metrics.insert("fault_type".into(), fault_type as u64 as f64);
            metrics.insert("injection_time".into(), chrono::Utc::now().timestamp() as f64);
            sim.metrics = metrics;
            warn!(
                simulation = %simulation_id,
                fault = ?fault_type,
                "Fault injected into simulation"
            );
        }
    }

    pub async fn record_result(&self, result: ExperimentResult) {
        info!(
            experiment = %result.experiment_id,
            passed = %result.passed,
            survived = %result.system_survived,
            "Chaos experiment result recorded"
        );
        self.results.write().await.push(result);
    }

    pub async fn get_experiments(&self) -> Vec<ChaosExperiment> {
        self.experiments.read().await.clone()
    }

    pub async fn get_results(&self) -> Vec<ExperimentResult> {
        self.results.read().await.clone()
    }

    pub async fn get_active_simulations(&self) -> Vec<SimulationInstance> {
        self.active_simulations.read().await.clone()
    }

    pub async fn verify_system_integrity_after_chaos(&self) -> SystemIntegrityReport {
        let results = self.results.read().await;
        let total = results.len() as u64;
        let passed = results.iter().filter(|r| r.passed).count() as u64;
        let survived = results.iter().filter(|r| r.system_survived).count() as u64;
        let data_ok = results.iter().filter(|r| r.data_integrity_verified).count() as u64;

        let failure_types: Vec<String> = results.iter()
            .filter(|r| !r.passed)
            .flat_map(|r| r.failures.clone())
            .collect();

        SystemIntegrityReport {
            total_experiments: total,
            passed,
            system_survived: survived,
            data_integrity_verified: data_ok,
            integrity_score: if total > 0 { (passed + survived + data_ok) as f64 / (total * 3) as f64 } else { 1.0 },
            failure_patterns: failure_types,
        }
    }
}

#[derive(Debug)]
pub struct SystemIntegrityReport {
    pub total_experiments: u64,
    pub passed: u64,
    pub system_survived: u64,
    pub data_integrity_verified: u64,
    pub integrity_score: f64,
    pub failure_patterns: Vec<String>,
}

impl Default for ChaosEngine {
    fn default() -> Self {
        Self::new()
    }
}
