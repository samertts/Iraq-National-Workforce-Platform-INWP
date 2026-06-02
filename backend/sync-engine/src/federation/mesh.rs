use super::topology::FederationTopology;
use super::{FederationDomain, FederationRoute, SovereigntyBoundary};
use crate::error::{SyncEngineError, SyncResult};
use std::collections::HashMap;
use tracing::{info, warn};

/// Routes events across the sovereign federation mesh
pub struct FederationMesh {
    local_domain: FederationDomain,
    topology: FederationTopology,
    route_table: HashMap<uuid::Uuid, FederationRoute>,
    peer_domains: HashMap<uuid::Uuid, FederationDomain>,
}

impl FederationMesh {
    pub fn new(local_domain: FederationDomain, topology: FederationTopology) -> Self {
        Self {
            local_domain,
            topology,
            route_table: HashMap::new(),
            peer_domains: HashMap::new(),
        }
    }

    pub fn register_peer(&mut self, domain: FederationDomain, route: FederationRoute) {
        let target_domain = route.target_domain;
        let route_type = route.route_type;
        self.peer_domains.insert(domain.domain_id, domain);
        self.route_table.insert(route.target_domain, route);
        info!(
            peer_domain = %target_domain,
            route_type = ?route_type,
            "Federation peer registered"
        );
    }

    pub fn unregister_peer(&mut self, domain_id: &uuid::Uuid) {
        self.peer_domains.remove(domain_id);
        self.route_table.remove(domain_id);
        warn!(domain_id = %domain_id, "Federation peer unregistered");
    }

    pub fn should_route_event(
        &self,
        event_type: &str,
        origin_domain: &uuid::Uuid,
    ) -> Vec<uuid::Uuid> {
        let mut targets = Vec::new();

        for domain_id in self.route_table.keys() {
            let peer = match self.peer_domains.get(domain_id) {
                Some(p) => p,
                None => continue,
            };

            if self.crosses_boundary(origin_domain, &peer.sovereignty_boundary, event_type) {
                targets.push(*domain_id);
            }
        }

        targets
    }

    pub fn routing_path(
        &self,
        source: &uuid::Uuid,
        target: &uuid::Uuid,
    ) -> SyncResult<Vec<uuid::Uuid>> {
        let path = self.topology.shortest_path(source, target)
            .ok_or_else(|| SyncEngineError::Internal(format!(
                "No routing path between domains {} and {}",
                source, target
            )))?;
        Ok(path)
    }

    pub fn local_domain(&self) -> &FederationDomain {
        &self.local_domain
    }

    pub fn peers(&self) -> &HashMap<uuid::Uuid, FederationDomain> {
        &self.peer_domains
    }

    pub fn has_route_to(&self, domain_id: &uuid::Uuid) -> bool {
        self.route_table.contains_key(domain_id)
    }

    fn crosses_boundary(
        &self,
        origin: &uuid::Uuid,
        boundary: &SovereigntyBoundary,
        event_type: &str,
    ) -> bool {
        if origin == &boundary.domain_id {
            return true;
        }

        let is_allowed = boundary.allow_inbound_schemas.is_empty()
            || boundary.allow_inbound_schemas.iter().any(|s| event_type.contains(s));

        let needs_approval = boundary.require_approval_for.iter().any(|s| event_type.contains(s));

        is_allowed && !needs_approval
    }
}
