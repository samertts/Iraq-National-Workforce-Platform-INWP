use crate::config::TransportConfig;
use crate::core::node::NodeType;
use crate::error::SyncResult;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, debug};

pub struct MeshDiscovery {
    node_id: uuid::Uuid,
    node_type: NodeType,
    region: String,
    config: TransportConfig,
    discovered_nodes: Arc<RwLock<HashMap<uuid::Uuid, DiscoveredNode>>>,
    running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub struct DiscoveredNode {
    pub node_id: uuid::Uuid,
    pub node_type: NodeType,
    pub address: String,
    pub port: u16,
    pub region: String,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub service_name: String,
}

impl MeshDiscovery {
    pub fn new(
        node_id: uuid::Uuid,
        node_type: NodeType,
        region: &str,
        config: &TransportConfig,
    ) -> Self {
        Self {
            node_id,
            node_type,
            region: region.to_string(),
            config: config.clone(),
            discovered_nodes: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&self) -> SyncResult<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        info!(
            node_id = %self.node_id,
            service = %self.config.mdns_service_name,
            "Starting mDNS discovery"
        );

        let discovered = self.discovered_nodes.clone();
        let region = self.region.clone();
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            while *running_flag.read().await {
                if let Err(e) = Self::discovery_cycle(&discovered, &region).await {
                    debug!("Discovery cycle error: {}", e);
                }
                tokio::time::sleep(Duration::from_secs(30)).await;
            }
        });

        Ok(())
    }

    async fn discovery_cycle(
        discovered: &Arc<RwLock<HashMap<uuid::Uuid, DiscoveredNode>>>,
        _region: &str,
    ) -> SyncResult<()> {
        let stale_threshold = chrono::Utc::now() - chrono::Duration::seconds(120);
        let mut nodes = discovered.write().await;
        nodes.retain(|_, n| n.last_seen > stale_threshold);
        Ok(())
    }

    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("mDNS discovery stopped");
    }

    pub async fn get_peers(&self) -> Vec<DiscoveredNode> {
        let nodes = self.discovered_nodes.read().await;
        nodes.values().cloned().collect()
    }

    pub async fn register_peer(&self, node: DiscoveredNode) {
        let mut nodes = self.discovered_nodes.write().await;
        nodes.insert(node.node_id, node);
    }

    pub async fn remove_peer(&self, node_id: &uuid::Uuid) {
        let mut nodes = self.discovered_nodes.write().await;
        nodes.remove(node_id);
    }
}
