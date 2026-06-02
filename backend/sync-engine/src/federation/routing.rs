use crate::error::{SyncEngineError, SyncResult};
use std::collections::{HashMap, VecDeque};

use super::{FederationDomain, FederationTier};

/// Hierarchical routing engine for sovereign federation topology
pub struct FederationRouter {
    domains: HashMap<uuid::Uuid, FederationDomain>,
    adjacency: HashMap<uuid::Uuid, Vec<uuid::Uuid>>,
}

impl Default for FederationRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl FederationRouter {
    pub fn new() -> Self {
        Self {
            domains: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }

    pub fn register_domain(&mut self, domain: FederationDomain) {
        let domain_id = domain.domain_id;

        if let Some(parent) = &domain.parent_domain {
            self.adjacency.entry(domain_id).or_default().push(*parent);
            self.adjacency.entry(*parent).or_default().push(domain_id);
        }

        self.domains.insert(domain_id, domain);
    }

    pub fn find_upstream(&self, domain_id: &uuid::Uuid) -> Option<&FederationDomain> {
        let domain = self.domains.get(domain_id)?;
        let parent_id = domain.parent_domain.as_ref()?;
        self.domains.get(parent_id)
    }

    pub fn find_downstream(&self, domain_id: &uuid::Uuid) -> Vec<&FederationDomain> {
        self.domains.values()
            .filter(|d| d.parent_domain.as_ref() == Some(domain_id))
            .collect()
    }

    pub fn find_siblings(&self, domain_id: &uuid::Uuid) -> Vec<&FederationDomain> {
        let domain = match self.domains.get(domain_id) {
            Some(d) => d,
            None => return Vec::new(),
        };

        let parent = match &domain.parent_domain {
            Some(p) => p,
            None => return Vec::new(),
        };

        self.domains.values()
            .filter(|d| d.parent_domain.as_ref() == Some(parent) && d.domain_id != *domain_id)
            .collect()
    }

    pub fn route_to_tier(
        &self,
        from: &uuid::Uuid,
        target_tier: FederationTier,
    ) -> SyncResult<Vec<uuid::Uuid>> {
        let path = self.path_to_tier(from, target_tier)
            .ok_or_else(|| SyncEngineError::Internal(format!(
                "No route from {} to tier {:?}",
                from, target_tier
            )))?;
        Ok(path)
    }

    pub fn broadcast_to_tier(
        &self,
        from: &uuid::Uuid,
        target_tier: FederationTier,
    ) -> Vec<uuid::Uuid> {
        self.domains.values()
            .filter(|d| d.tier == target_tier)
            .map(|d| d.domain_id)
            .filter(|id| id != from)
            .collect()
    }

    pub fn domain_count_by_tier(&self, tier: FederationTier) -> usize {
        self.domains.values().filter(|d| d.tier == tier).count()
    }

    pub fn all_domains(&self) -> &HashMap<uuid::Uuid, FederationDomain> {
        &self.domains
    }

    fn path_to_tier(&self, from: &uuid::Uuid, target_tier: FederationTier) -> Option<Vec<uuid::Uuid>> {
        // BFS to find shortest path to any domain of target_tier
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: HashMap<uuid::Uuid, uuid::Uuid> = HashMap::new();

        visited.insert(*from);
        queue.push_back(*from);

        while let Some(current) = queue.pop_front() {
            if let Some(domain) = self.domains.get(&current) {
                if domain.tier == target_tier && current != *from {
                    // Reconstruct path
                    let mut path = Vec::new();
                    let mut node = current;
                    while node != *from {
                        path.push(node);
                        node = *parent.get(&node)?;
                    }
                    path.reverse();
                    return Some(path);
                }
            }

            if let Some(neighbors) = self.adjacency.get(&current) {
                for &neighbor in neighbors {
                    if visited.insert(neighbor) {
                        parent.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        None
    }
}

use std::collections::HashSet;
