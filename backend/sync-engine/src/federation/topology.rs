use super::{FederationDomain, FederationTier};
use std::collections::HashMap;

/// Directed acyclic graph of sovereign federation topology
pub struct FederationTopology {
    nodes: HashMap<uuid::Uuid, TopologyNode>,
    edges: Vec<TopologyEdge>,
}

struct TopologyNode {
    domain_id: uuid::Uuid,
    tier: FederationTier,
    parent: Option<uuid::Uuid>,
    children: Vec<uuid::Uuid>,
}

struct TopologyEdge {
    source: uuid::Uuid,
    target: uuid::Uuid,
    weight: u32,
}

impl Default for FederationTopology {
    fn default() -> Self {
        Self::new()
    }
}

impl FederationTopology {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_domain(&mut self, domain: &FederationDomain) {
        let node = TopologyNode {
            domain_id: domain.domain_id,
            tier: domain.tier,
            parent: domain.parent_domain,
            children: Vec::new(),
        };
        self.nodes.insert(domain.domain_id, node);

        // Link to parent
        if let Some(parent_id) = domain.parent_domain {
            if let Some(parent) = self.nodes.get_mut(&parent_id) {
                parent.children.push(domain.domain_id);
            }
            self.edges.push(TopologyEdge {
                source: parent_id,
                target: domain.domain_id,
                weight: 1,
            });
        }
    }

    pub fn shortest_path(&self, from: &uuid::Uuid, to: &uuid::Uuid) -> Option<Vec<uuid::Uuid>> {
        if from == to {
            return Some(vec![*from]);
        }

        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut parent: HashMap<uuid::Uuid, uuid::Uuid> = HashMap::new();

        visited.insert(*from);
        queue.push_back(*from);

        while let Some(current) = queue.pop_front() {
            if current == *to {
                let mut path = Vec::new();
                let mut node = current;
                while node != *from {
                    path.push(node);
                    node = *parent.get(&node)?;
                }
                path.push(*from);
                path.reverse();
                return Some(path);
            }

            let neighbors = self.get_neighbors(&current);
            for neighbor in neighbors {
                if visited.insert(neighbor) {
                    parent.insert(neighbor, current);
                    queue.push_back(neighbor);
                }
            }
        }

        None
    }

    pub fn subtree(&self, root_id: &uuid::Uuid) -> Vec<uuid::Uuid> {
        let mut result = Vec::new();
        let mut stack = vec![*root_id];

        while let Some(current) = stack.pop() {
            result.push(current);
            if let Some(node) = self.nodes.get(&current) {
                for child in &node.children {
                    stack.push(*child);
                }
            }
        }

        result
    }

    pub fn depth(&self, domain_id: &uuid::Uuid) -> Option<u32> {
        let mut depth = 0;
        let mut current = domain_id;

        loop {
            let node = self.nodes.get(current)?;
            match &node.parent {
                Some(parent) => {
                    depth += 1;
                    current = parent;
                }
                None => return Some(depth),
            }
        }
    }

    fn get_neighbors(&self, domain_id: &uuid::Uuid) -> Vec<uuid::Uuid> {
        let mut neighbors = Vec::new();

        // Add parent
        if let Some(node) = self.nodes.get(domain_id) {
            if let Some(parent) = node.parent {
                neighbors.push(parent);
            }
            // Add children
            neighbors.extend(&node.children);
        }

        // Add edge-based neighbors
        for edge in &self.edges {
            if edge.source == *domain_id {
                neighbors.push(edge.target);
            } else if edge.target == *domain_id {
                neighbors.push(edge.source);
            }
        }

        neighbors
    }
}
