use crate::config::SyncEngineConfig;
use crate::core::{Capabilities, NodeIdentity, NodeStatus};
use crate::error::{SyncEngineError, SyncResult};
use crate::events::contract::SyncEvent;
use crate::storage::PgStore;
use futures::StreamExt;
use std::sync::Arc;
use tracing::{error, info, warn};

pub async fn run(config: SyncEngineConfig, store: Arc<PgStore>) -> SyncResult<()> {
    let Some(url) = config.transport.nats_url.clone() else {
        info!("NATS bridge disabled: NATS_URL is not configured");
        return Ok(());
    };
    let client = async_nats::connect(&url)
        .await
        .map_err(|error| SyncEngineError::Nats(Box::new(error)))?;
    let mut subscription = client
        .subscribe("inwp.sync.event.>")
        .await
        .map_err(|error| SyncEngineError::Nats(Box::new(error)))?;
    let local_node = NodeIdentity {
        node_id: store_node_id(&config),
        node_type: (&config.node.node_type).into(),
        node_name: config.node.node_name.clone(),
        ministry_id: uuid::Uuid::parse_str(&config.node.ministry_id)
            .unwrap_or_else(|_| uuid::Uuid::nil()),
        site_id: uuid::Uuid::parse_str(&config.node.site_id).unwrap_or_else(|_| uuid::Uuid::nil()),
        region: config.node.region.clone(),
        certificate_serial: String::new(),
        public_key: Vec::new(),
        address: config.transport.grpc_listen.clone(),
        port: config.transport.grpc_port,
        capabilities: Capabilities {
            schema_versions: vec!["1.0".into()],
            supported_entities: vec!["clock_event".into()],
            max_partitions: 1024,
            max_batch_size_bytes: config.protocol.max_batch_bytes,
            supports_compression: true,
            compression_algorithms: vec!["zstd".into()],
            supports_lan_mesh: config.transport.mdns_enabled,
        },
        status: NodeStatus::Online,
        last_heartbeat: chrono::Utc::now(),
    };
    store.nodes.upsert(&local_node).await?;
    info!(subject = "inwp.sync.event.>", "NATS sync bridge subscribed");

    while let Some(message) = subscription.next().await {
        let event: SyncEvent = match serde_json::from_slice(&message.payload) {
            Ok(event) => event,
            Err(error) => {
                warn!(error = %error, "discarding malformed sync event");
                continue;
            }
        };
        if event.payload.len() as u64 > config.events.max_event_size_bytes {
            warn!(event_id = %event.event_id, "discarding oversized sync event");
            continue;
        }
        if let Err(error) = store.events.append(&event).await {
            error!(event_id = %event.event_id, error = %error, "failed to persist sync event");
            continue;
        }
        let event_id = event.event_id;
        if let Err(error) = store
            .queue
            .enqueue(
                store_node_id(&config),
                &event.partition_key,
                event_id,
                &event.event_type,
                &event.payload,
                5,
            )
            .await
        {
            error!(event_id = %event_id, error = %error, "failed to enqueue sync event");
        }
    }
    Ok(())
}

fn store_node_id(config: &SyncEngineConfig) -> uuid::Uuid {
    uuid::Uuid::parse_str(&config.node.node_id).unwrap_or_else(|_| uuid::Uuid::nil())
}
