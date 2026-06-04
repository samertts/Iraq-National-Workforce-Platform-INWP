use std::collections::{HashMap, HashSet, VecDeque};
use tracing::info;

/// Dependency node in the architecture graph
#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub name: String,
    pub dependencies: HashSet<String>,
    pub dependents: HashSet<String>,
    pub metadata: HashMap<String, String>,
}

/// Result of a coupling analysis
#[derive(Debug)]
pub struct CouplingReport {
    pub component: String,
    pub coupling_score: f64,
    pub afferent_coupling: usize,
    pub efferent_coupling: usize,
    pub instability: f64,
    pub abstractness: f64,
    pub distance_from_main: f64,
    pub violations: Vec<String>,
}

/// Cycle detected in the dependency graph
#[derive(Debug, Clone)]
pub struct DependencyCycle {
    pub nodes: Vec<String>,
    pub length: usize,
}

/// Structural analysis of the architecture
pub struct DependencyAnalyzer {
    nodes: HashMap<String, DependencyNode>,
    cycles: Vec<DependencyCycle>,
    analysis_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            cycles: Vec::new(),
            analysis_timestamp: None,
        }
    }

    pub fn register_component(&mut self, name: &str, metadata: HashMap<String, String>) {
        self.nodes.entry(name.to_string()).or_insert_with(|| {
            info!(component = %name, "Component registered for dependency analysis");
            DependencyNode {
                name: name.to_string(),
                dependencies: HashSet::new(),
                dependents: HashSet::new(),
                metadata,
            }
        });
    }

    pub fn register_dependency(&mut self, source: &str, target: &str) {
        let source_name = source.to_string();
        let target_name = target.to_string();

        if source_name == target_name {
            return;
        }

        let src = self
            .nodes
            .entry(source_name.clone())
            .or_insert_with(|| DependencyNode {
                name: source_name.clone(),
                dependencies: HashSet::new(),
                dependents: HashSet::new(),
                metadata: HashMap::new(),
            });
        src.dependencies.insert(target_name.clone());

        let tgt = self
            .nodes
            .entry(target_name.clone())
            .or_insert_with(|| DependencyNode {
                name: target_name.clone(),
                dependencies: HashSet::new(),
                dependents: HashSet::new(),
                metadata: HashMap::new(),
            });
        tgt.dependents.insert(source_name);
    }

    pub fn has_dependency(&self, source: &str, target: &str) -> bool {
        self.nodes
            .get(source)
            .map(|n| n.dependencies.contains(target))
            .unwrap_or(false)
    }

    pub fn compute_coupling_score(&self, component: &str) -> f64 {
        let report = self.analyze_coupling(component);
        report.coupling_score
    }

    pub fn analyze_coupling(&self, component: &str) -> CouplingReport {
        let efferent = self
            .nodes
            .get(component)
            .map(|n| n.dependencies.len())
            .unwrap_or(0);
        let afferent = self
            .nodes
            .get(component)
            .map(|n| n.dependents.len())
            .unwrap_or(0);

        let total = efferent + afferent;
        let instability = if total > 0 {
            efferent as f64 / total as f64
        } else {
            0.0
        };

        let abstractness = 0.0;
        let distance = (abstractness + instability - 1.0).abs();

        let coupling_score = instability * 0.4 + (1.0 - abstractness) * 0.3 + distance * 0.3;

        let mut violations = Vec::new();
        if instability > 0.7 {
            violations.push(format!(
                "High instability ({:.2}): component depends on many others",
                instability
            ));
        }
        if afferent > 20 {
            violations.push(format!(
                "High afferent coupling ({}): too many dependents",
                afferent
            ));
        }
        if efferent > 15 {
            violations.push(format!(
                "High efferent coupling ({}): too many dependencies",
                efferent
            ));
        }
        if distance > 0.5 {
            violations.push(format!(
                "Far from main sequence ({:.2}): abstractness/instability imbalance",
                distance
            ));
        }

        CouplingReport {
            component: component.to_string(),
            coupling_score,
            afferent_coupling: afferent,
            efferent_coupling: efferent,
            instability,
            abstractness,
            distance_from_main: distance,
            violations,
        }
    }

    pub fn detect_cycles(&mut self) -> Vec<DependencyCycle> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut in_stack = HashSet::new();
        let mut path: Vec<String> = Vec::new();
        let nodes: Vec<String> = self.nodes.keys().cloned().collect();

        for node in &nodes {
            if visited.insert(node.clone()) {
                self.dfs_cycle(node, &mut visited, &mut in_stack, &mut path, &mut cycles);
            }
        }

        self.cycles = cycles.clone();
        self.analysis_timestamp = Some(chrono::Utc::now());

        if !cycles.is_empty() {
            info!(
                count = cycles.len(),
                "Dependency cycles detected in architecture"
            );
            for cycle in &cycles {
                info!(length = cycle.length, nodes = ?cycle.nodes, "Cycle detected");
            }
        }

        cycles
    }

    fn dfs_cycle(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        in_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<DependencyCycle>,
    ) {
        in_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(n) = self.nodes.get(node) {
            for dep in &n.dependencies {
                if in_stack.contains(dep) {
                    let cycle_start = path.iter().position(|n| n == dep).unwrap();
                    let cycle_nodes: Vec<String> = path[cycle_start..].to_vec();
                    cycles.push(DependencyCycle {
                        length: cycle_nodes.len(),
                        nodes: cycle_nodes,
                    });
                } else if !visited.contains(dep) {
                    visited.insert(dep.clone());
                    self.dfs_cycle(dep, visited, in_stack, path, cycles);
                }
            }
        }

        path.pop();
        in_stack.remove(node);
    }

    pub fn compute_impact_analysis(&self, changed_component: &str) -> ImpactAnalysis {
        let mut affected = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(changed_component.to_string());

        while let Some(component) = queue.pop_front() {
            if let Some(node) = self.nodes.get(&component) {
                for dependent in &node.dependents {
                    if affected.insert(dependent.clone()) {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        let mut direct = Vec::new();
        let mut transitive = Vec::new();

        if let Some(node) = self.nodes.get(changed_component) {
            for dep in &node.dependencies {
                direct.push(dep.clone());
                let ta = self.compute_impact_analysis(dep);
                transitive.extend(ta.directly_affected);
                transitive.extend(ta.transitive_dependencies);
            }
        }

        ImpactAnalysis {
            changed_component: changed_component.to_string(),
            directly_affected: affected
                .iter()
                .filter(|c| *c != changed_component)
                .cloned()
                .collect(),
            transitive_dependencies: transitive,
            total_affected: affected.len(),
        }
    }

    pub fn generate_architecture_report(&self) -> ArchitectureReport {
        let mut report = ArchitectureReport {
            total_components: self.nodes.len(),
            total_cycles: self.cycles.len(),
            average_coupling: 0.0,
            most_coupled: None,
            most_instable: None,
            cycle_details: self.cycles.clone(),
            component_reports: Vec::new(),
        };

        let mut total_coupling = 0.0;
        let mut max_coupling = 0.0;
        let mut max_instability = 0.0;

        for name in self.nodes.keys() {
            let coupling = self.analyze_coupling(name);
            total_coupling += coupling.coupling_score;
            if coupling.coupling_score > max_coupling {
                max_coupling = coupling.coupling_score;
                report.most_coupled = Some(name.clone());
            }
            if coupling.instability > max_instability {
                max_instability = coupling.instability;
                report.most_instable = Some(name.clone());
            }
            report.component_reports.push(coupling);
        }

        report.average_coupling = if report.total_components > 0 {
            total_coupling / report.total_components as f64
        } else {
            0.0
        };

        report
    }
}

#[derive(Debug)]
pub struct ImpactAnalysis {
    pub changed_component: String,
    pub directly_affected: Vec<String>,
    pub transitive_dependencies: Vec<String>,
    pub total_affected: usize,
}

#[derive(Debug)]
pub struct ArchitectureReport {
    pub total_components: usize,
    pub total_cycles: usize,
    pub average_coupling: f64,
    pub most_coupled: Option<String>,
    pub most_instable: Option<String>,
    pub cycle_details: Vec<DependencyCycle>,
    pub component_reports: Vec<CouplingReport>,
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
