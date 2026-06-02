use sync_engine::config::SyncEngineConfig;
use sync_engine::observability::init_observability;
use sync_engine::storage::PgStore;
use sync_engine::transport::grpc::GrpcServer;
use sync_engine::transport::mesh::MeshDiscovery;
use sync_engine::core::node::NodeType;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = SyncEngineConfig::from_env()?;

    let _guard = init_observability(&config.observability)?;

    info!(
        node_id = %config.node.node_id,
        node_type = %config.node.node_type.as_str(),
        region = %config.node.region,
        "Starting sync engine"
    );

    let store = PgStore::new(&config.storage).await?;
    let pool = store.pool.clone();

    let node_id = uuid::Uuid::parse_str(&config.node.node_id)
        .map_err(|e| anyhow::anyhow!("Invalid node_id: {}", e))?;

    let node_type: NodeType = (&config.node.node_type).into();

    let mesh_discovery = Arc::new(
        MeshDiscovery::new(
            node_id,
            node_type,
            &config.node.region,
            &config.transport,
        )
    );

    if config.transport.mdns_enabled {
        let md = mesh_discovery.clone();
        tokio::spawn(async move {
            if let Err(e) = md.start().await {
                error!(error = %e, "mDNS discovery failed");
            }
        });
    }

    let grpc_server = GrpcServer::new(
        config.clone(),
        pool.clone(),
        mesh_discovery.clone(),
    );

    let grpc_addr = format!("{}:{}", config.transport.grpc_listen, config.transport.grpc_port);
    info!(address = %grpc_addr, "Starting gRPC sync server");

    let _grpc_handle = tokio::spawn(async move {
        if let Err(e) = grpc_server.serve(&grpc_addr).await {
            error!(error = %e, "gRPC server failed");
        }
    });

    info!("Sync engine started successfully");

    signal::ctrl_c().await?;
    info!("Shutdown signal received");

    mesh_discovery.stop().await;
    info!("Shutdown complete");

    Ok(())
}
