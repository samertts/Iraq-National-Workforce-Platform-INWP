use std::sync::Arc;
use sync_engine::config::SyncEngineConfig;
use sync_engine::core::node::NodeType;
use sync_engine::observability::init_observability;
use sync_engine::storage::PgStore;
use sync_engine::transport::grpc::GrpcServer;
use sync_engine::transport::mesh::MeshDiscovery;
use sync_engine::transport::nats_bridge;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{error, info};

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

    let store = Arc::new(PgStore::new(&config.storage).await?);
    let pool = store.pool.clone();

    if config.transport.nats_url.is_some() {
        let bridge_store = store.clone();
        let bridge_config = config.clone();
        tokio::spawn(async move {
            if let Err(error) = nats_bridge::run(bridge_config, bridge_store).await {
                error!(error = %error, "NATS sync bridge stopped");
            }
        });
    }

    let node_id = uuid::Uuid::parse_str(&config.node.node_id)
        .map_err(|e| anyhow::anyhow!("Invalid node_id: {}", e))?;

    let node_type: NodeType = (&config.node.node_type).into();

    let mesh_discovery = Arc::new(MeshDiscovery::new(
        node_id,
        node_type,
        &config.node.region,
        &config.transport,
    ));

    if config.transport.mdns_enabled {
        let md = mesh_discovery.clone();
        tokio::spawn(async move {
            if let Err(e) = md.start().await {
                error!(error = %e, "mDNS discovery failed");
            }
        });
    }

    let grpc_server = GrpcServer::new(config.clone(), pool.clone(), mesh_discovery.clone());

    let grpc_addr = format!(
        "{}:{}",
        config.transport.grpc_listen, config.transport.grpc_port
    );
    info!(address = %grpc_addr, "Starting gRPC sync server");

    let _grpc_handle = tokio::spawn(async move {
        if let Err(e) = grpc_server.serve(&grpc_addr).await {
            error!(error = %e, "gRPC server failed");
        }
    });

    let health_addr = format!("0.0.0.0:{}", config.observability.health_check_port);
    tokio::spawn(async move {
        if let Err(error) = run_health_server(&health_addr).await {
            error!(error = %error, "health server stopped");
        }
    });

    info!("Sync engine started successfully");

    signal::ctrl_c().await?;
    info!("Shutdown signal received");

    mesh_discovery.stop().await;
    info!("Shutdown complete");

    Ok(())
}

async fn run_health_server(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    info!(address = %addr, "Health server listening");
    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buffer = [0u8; 1024];
            let size = match socket.read(&mut buffer).await {
                Ok(size) => size,
                Err(_) => return,
            };
            let request = String::from_utf8_lossy(&buffer[..size]);
            let (status, body) =
                if request.starts_with("GET /readyz") || request.starts_with("GET /healthz") {
                    ("200 OK", r#"{"status":"ok","service":"sync-engine"}"#)
                } else {
                    ("404 Not Found", r#"{"status":"not_found"}"#)
                };
            let response = format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            let _ = socket.write_all(response.as_bytes()).await;
        });
    }
}
