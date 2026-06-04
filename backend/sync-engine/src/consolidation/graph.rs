use std::collections::{HashMap, HashSet, VecDeque};
use tracing::info;

/// Infrastructure knowledge graph — the unified representation of all platform topology,
/// dependencies, lineage, trust, governance, and sovereignty relationships.
pub struct KnowledgeGraph {
    nodes: HashMap<String, GraphNode>,
    edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone)]
pub struct GraphNode {
    pub node_id: String,
    pub node_type: GraphNodeType,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphNodeType {
    Domain,
    Service,
    Schema,
    Event,
    Federation,
    SovereigntyZone,
    GovernancePolicy,
    Deployment,
    TrustRelationship,
    Checkpoint,
}

#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
    pub edge_type: GraphEdgeType,
    pub weight: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphEdgeType {
    DependsOn,
    CommunicatesVia,
    RoutesTo,
    OwnedBy,
    GovernedBy,
    ReplicatesTo,
    Trusts,
    BelongsTo,
    DerivesFrom,
    ReplaysTo,
}

#[derive(Debug)]
pub struct ImpactPath {
    pub source: String,
    pub target: String,
    pub path: Vec<String>,
    pub total_affinity: f64,
    pub hops: u32,
}

#[derive(Debug)]
pub struct GraphAnalysis {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub strongly_connected_components: usize,
    pub average_path_length: f64,
    pub graph_density: f64,
    pub central_nodes: Vec<String>,
}

pub struct LineageGraph;

#[derive(Debug)]
pub struct LineagePath {
    pub source_event: String,
    pub target_state: String,
    pub path: Vec<LineageHop>,
    pub deterministic: bool,
}

#[derive(Debug)]
pub struct LineageHop {
    pub event_id: uuid::Uuid,
    pub transformation: String,
    pub domain: String,
}

impl KnowledgeGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: GraphNode) {
        let id = node.node_id.clone();
        info!(node = %id, kind = ?node.node_type, "Graph node registered");
        self.nodes.insert(id, node);
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        let src = edge.source.clone();
        let tgt = edge.target.clone();
        info!(source = %src, target = %tgt, kind = ?edge.edge_type, "Graph edge registered");
        self.edges.push(edge);
    }

    pub fn compute_impact(&self, source: &str) -> Vec<ImpactPath> {
        let mut paths = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back((source.to_string(), vec![source.to_string()], 1.0));

        while let Some((current, path, affinity)) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }
            for edge in &self.edges {
                if edge.source == current {
                    let mut new_path = path.clone();
                    new_path.push(edge.target.clone());
                    let new_affinity = affinity * edge.weight;
                    paths.push(ImpactPath {
                        source: source.to_string(),
                        target: edge.target.clone(),
                        path: new_path.clone(),
                        total_affinity: new_affinity,
                        hops: new_path.len() as u32 - 1,
                    });
                    queue.push_back((edge.target.clone(), new_path, new_affinity));
                }
            }
        }

        paths.sort_by(|a, b| {
            b.total_affinity
                .partial_cmp(&a.total_affinity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        paths
    }

    pub fn analyze(&self) -> GraphAnalysis {
        let total_nodes = self.nodes.len();
        let total_edges = self.edges.len();
        let density = if total_nodes > 1 {
            total_edges as f64 / (total_nodes * (total_nodes - 1)) as f64
        } else {
            0.0
        };

        let mut adjacency: HashMap<&str, HashSet<&str>> = HashMap::new();
        for edge in &self.edges {
            adjacency
                .entry(edge.source.as_str())
                .or_default()
                .insert(edge.target.as_str());
            adjacency
                .entry(edge.target.as_str())
                .or_default()
                .insert(edge.source.as_str());
        }

        let scc = self.count_strongly_connected();
        let central = self.find_central_nodes();

        GraphAnalysis {
            total_nodes,
            total_edges,
            strongly_connected_components: scc,
            average_path_length: 0.0,
            graph_density: density,
            central_nodes: central,
        }
    }

    fn count_strongly_connected(&self) -> usize {
        let node_ids: Vec<&String> = self.nodes.keys().collect();
        let mut visited = HashSet::new();
        let mut count = 0;

        for id in &node_ids {
            if visited.insert((*id).clone()) {
                count += 1;
                let mut stack = vec![(*id).clone()];
                while let Some(current) = stack.pop() {
                    for edge in &self.edges {
                        if edge.source == current && visited.insert(edge.target.clone()) {
                            stack.push(edge.target.clone());
                        }
                        if edge.target == current && visited.insert(edge.source.clone()) {
                            stack.push(edge.source.clone());
                        }
                    }
                }
            }
        }

        count
    }

    fn find_central_nodes(&self) -> Vec<String> {
        let mut degree: HashMap<&str, usize> = HashMap::new();
        for edge in &self.edges {
            *degree.entry(edge.source.as_str()).or_default() += 1;
            *degree.entry(edge.target.as_str()).or_default() += 1;
        }
        let max_degree = degree.values().cloned().max().unwrap_or(0);
        if max_degree == 0 {
            return Vec::new();
        }
        let threshold = (max_degree as f64 * 0.7) as usize;
        let mut central: Vec<String> = degree
            .into_iter()
            .filter(|(_, d)| *d >= threshold)
            .map(|(n, _)| n.to_string())
            .collect();
        central.sort();
        central
    }
}

impl LineageGraph {
    pub fn new() -> Self {
        Self
    }

    pub fn trace_lineage(
        &self,
        event_id: uuid::Uuid,
        transformations: Vec<(uuid::Uuid, String, String)>,
    ) -> LineagePath {
        let hops: Vec<LineageHop> = transformations
            .into_iter()
            .map(|(eid, transform, domain)| LineageHop {
                event_id: eid,
                transformation: transform,
                domain,
            })
            .collect();

        let deterministic = hops.iter().all(|h| h.transformation != "non_deterministic");

        LineagePath {
            source_event: event_id.to_string(),
            target_state: hops
                .last()
                .map(|h| h.event_id.to_string())
                .unwrap_or_default(),
            path: hops,
            deterministic,
        }
    }
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for LineageGraph {
    fn default() -> Self {
        Self::new()
    }
}
