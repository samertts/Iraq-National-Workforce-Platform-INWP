use crate::config::{NodeTypeConfig, SyncEngineConfig};
use crate::core::{
    Capabilities, Checkpoint, NodeIdentity, NodeStatus, NodeType, RecordOperation, SyncRecord,
    VersionVector,
};
use crate::error::SyncResult;
use crate::events::contract::SyncEvent;
use crate::protocol::SyncSession;
use crate::recovery::state::{RecoveryPhase, RecoveryStateMachine};
use crate::storage::{CheckpointRepo, ConflictRepo, EventStore, NodeRepo, Pool, QueueRepo};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{
    transport::{Certificate, Identity, Server, ServerTlsConfig},
    Request, Response, Status,
};
use tracing::info;

pub mod proto {
    tonic::include_proto!("inwp.sync.v1");
}

use proto::sync_service_server::{SyncService, SyncServiceServer};
use proto::{
    sync_request, sync_response, ApplyDeltaRequest, ApplyDeltaResponse,
    Capabilities as ProtoCapabilities, DeltaRequest, DeltaResponse, DiscoveryResponse, Heartbeat,
    HeartbeatAck, MerkleDiffResponse, MerkleRootRequest, MerkleRootResponse,
    NodeIdentity as ProtoNodeIdentity, NodeInfoRequest, PeersRequest, PeersResponse,
    RecoveryStateRequest, RecoveryStateResponse, ReplayEvent, ReplayRequest,
    ResolveConflictRequest, ResolveConflictResponse, StatusRequest, StatusResponse, SyncCheckpoint,
    SyncRecord as ProtoSyncRecord, SyncRequest, SyncResponse,
};

#[derive(Clone)]
pub struct GrpcServer {
    config: SyncEngineConfig,
    pool: Pool,
    mesh_discovery: Arc<crate::transport::mesh::MeshDiscovery>,
    sessions: Arc<RwLock<HashMap<uuid::Uuid, SyncSession>>>,
}

impl GrpcServer {
    pub fn new(
        config: SyncEngineConfig,
        pool: Pool,
        mesh_discovery: Arc<crate::transport::mesh::MeshDiscovery>,
    ) -> Self {
        Self {
            config,
            pool,
            mesh_discovery,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn serve(self, addr: &str) -> SyncResult<()> {
        let addr = addr.parse().map_err(|e| {
            crate::error::SyncEngineError::Transport(format!("Invalid address: {}", e))
        })?;
        let sync_service = SyncServiceImpl::new(
            self.config.clone(),
            self.pool.clone(),
            self.sessions.clone(),
        );
        info!("gRPC SyncService listening on {}", addr);
        let mut server = Server::builder();
        let cert_path = &self.config.security.tls_cert_path;
        let key_path = &self.config.security.tls_key_path;
        let ca_path = &self.config.security.tls_ca_path;
        let tls_files_exist = std::path::Path::new(cert_path).exists()
            && std::path::Path::new(key_path).exists()
            && std::path::Path::new(ca_path).exists();
        if tls_files_exist {
            let cert = std::fs::read(cert_path).map_err(|e| {
                crate::error::SyncEngineError::Crypto(format!("read TLS cert: {}", e))
            })?;
            let key = std::fs::read(key_path).map_err(|e| {
                crate::error::SyncEngineError::Crypto(format!("read TLS key: {}", e))
            })?;
            let mut tls = ServerTlsConfig::new().identity(Identity::from_pem(cert, key));
            if self.config.security.mtls_required {
                let ca = std::fs::read(ca_path).map_err(|e| {
                    crate::error::SyncEngineError::Crypto(format!("read TLS CA: {}", e))
                })?;
                tls = tls.client_ca_root(Certificate::from_pem(ca));
            }
            server = server.tls_config(tls).map_err(|e| {
                crate::error::SyncEngineError::Transport(format!("TLS configuration failed: {}", e))
            })?;
        } else if self.config.security.mtls_required {
            return Err(crate::error::SyncEngineError::Crypto(
                "mTLS is required but certificate files are missing".into(),
            ));
        } else {
            tracing::warn!(
                "TLS certificates are not configured; serving gRPC without TLS in development mode"
            );
        }
        server
            .add_service(SyncServiceServer::new(sync_service))
            .serve(addr)
            .await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct SyncServiceImpl {
    config: SyncEngineConfig,
    events: EventStore,
    nodes: NodeRepo,
    checkpoints: CheckpointRepo,
    conflicts: ConflictRepo,
    queue: QueueRepo,
    sessions: Arc<RwLock<HashMap<uuid::Uuid, SyncSession>>>,
    recovery: Arc<RwLock<RecoveryStateMachine>>,
}

impl SyncServiceImpl {
    pub fn new(
        config: SyncEngineConfig,
        pool: Pool,
        sessions: Arc<RwLock<HashMap<uuid::Uuid, SyncSession>>>,
    ) -> Self {
        let local_uuid =
            uuid::Uuid::parse_str(&config.node.node_id).unwrap_or_else(|_| uuid::Uuid::nil());
        let region = config.node.region.clone();
        Self {
            config,
            events: EventStore::new(pool.clone()),
            nodes: NodeRepo::new(pool.clone()),
            checkpoints: CheckpointRepo::new(pool.clone()),
            conflicts: ConflictRepo::new(pool.clone()),
            queue: QueueRepo::new(pool),
            sessions,
            recovery: Arc::new(RwLock::new(RecoveryStateMachine::new(local_uuid, region))),
        }
    }

    fn local_uuid(&self) -> uuid::Uuid {
        uuid::Uuid::parse_str(&self.config.node.node_id).unwrap_or_else(|_| uuid::Uuid::nil())
    }

    fn local_node(&self) -> ProtoNodeIdentity {
        ProtoNodeIdentity {
            node_id: self.local_uuid().as_bytes().to_vec(),
            node_type: match self.config.node.node_type {
                NodeTypeConfig::NationalHub => proto::NodeType::NationalHub as i32,
                NodeTypeConfig::RegionalRelay => proto::NodeType::RegionalRelay as i32,
                NodeTypeConfig::Edge => proto::NodeType::Edge as i32,
                NodeTypeConfig::Mobile => proto::NodeType::Mobile as i32,
                NodeTypeConfig::DrReplica => proto::NodeType::DrReplica as i32,
            },
            node_name: self.config.node.node_name.clone(),
            ministry_id: self.config.node.ministry_id.clone(),
            site_id: self.config.node.site_id.clone(),
            region: self.config.node.region.clone(),
            certificate_serial: String::new(),
            public_key: Vec::new(),
            address: self.config.transport.grpc_listen.clone(),
            port: self.config.transport.grpc_port as u32,
            capabilities: Some(ProtoCapabilities {
                schema_versions: vec!["1.0".into()],
                supported_entities: vec!["clock_event".into()],
                max_partitions: 1024,
                max_batch_size_bytes: self.config.protocol.max_batch_bytes,
                supports_compression: true,
                compression_algorithms: vec!["zstd".into()],
                supports_lan_mesh: self.config.transport.mdns_enabled,
            }),
            status: proto::NodeStatus::Online as i32,
            last_heartbeat: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
        }
    }

    async fn merkle(&self, partition_key: &str) -> Result<(Vec<u8>, u64, u32), Status> {
        let records = self
            .events
            .fetch_records(partition_key, &[], 0, i64::MAX, true)
            .await
            .map_err(internal)?;
        let mut tree = crate::core::MerkleTree::new(partition_key);
        for record in &records {
            tree.insert(&record.record_id, &record.payload);
        }
        Ok((tree.root_hash, tree.leaf_count as u64, tree.height))
    }

    fn checkpoint_proto(checkpoint: Checkpoint) -> SyncCheckpoint {
        SyncCheckpoint {
            node_id: checkpoint.node_id.as_bytes().to_vec(),
            partition_key: checkpoint.partition_key,
            merkle_root: checkpoint.merkle_root,
            last_sync_at: Some(to_timestamp(&checkpoint.last_sync_at)),
            synced_events: checkpoint.synced_events,
            last_error: checkpoint.last_error.unwrap_or_default(),
        }
    }
}

#[tonic::async_trait]
impl SyncService for SyncServiceImpl {
    type InitiateSyncStream = ReceiverStream<Result<SyncResponse, Status>>;
    type GetDeltaStream = ReceiverStream<Result<DeltaResponse, Status>>;
    type InitiateReplayStream = ReceiverStream<Result<ReplayEvent, Status>>;

    async fn initiate_sync(
        &self,
        request: Request<tonic::Streaming<SyncRequest>>,
    ) -> Result<Response<Self::InitiateSyncStream>, Status> {
        let mut inbound = request.into_inner();
        let (tx, rx) = mpsc::channel(16);
        let service = self.clone();
        tokio::spawn(async move {
            loop {
                let message = match inbound.message().await {
                    Ok(Some(message)) => message,
                    Ok(None) => break,
                    Err(error) => {
                        let _ = tx.send(Err(Status::internal(error.to_string()))).await;
                        break;
                    }
                };
                let session_id = if message.session_id.len() == 16 {
                    message.session_id.clone()
                } else {
                    uuid::Uuid::now_v7().as_bytes().to_vec()
                };
                let response = match message.phase {
                    Some(sync_request::Phase::Discovery(request)) => {
                        let checkpoints = match service
                            .checkpoints
                            .list_for_node(service.local_uuid())
                            .await
                        {
                            Ok(items) => items.into_iter().map(Self::checkpoint_proto).collect(),
                            Err(error) => {
                                let _ = tx.send(Err(internal(error))).await;
                                continue;
                            }
                        };
                        SyncResponse {
                            session_id,
                            seq: message.seq,
                            error: String::new(),
                            phase: Some(sync_response::Phase::Discovery(DiscoveryResponse {
                                node: Some(service.local_node()),
                                partitions: request.partitions,
                                checkpoints,
                                accepted: true,
                                reject_reason: String::new(),
                                suggested_batch_size: service.config.protocol.max_batch_size,
                            })),
                        }
                    }
                    Some(sync_request::Phase::MerkleDiff(request)) => {
                        let (root, _, _) = match service.merkle(&request.partition_key).await {
                            Ok(value) => value,
                            Err(error) => {
                                let _ = tx.send(Err(error)).await;
                                continue;
                            }
                        };
                        SyncResponse {
                            session_id,
                            seq: message.seq,
                            error: String::new(),
                            phase: Some(sync_response::Phase::MerkleDiff(MerkleDiffResponse {
                                partition_key: request.partition_key,
                                local_merkle_root: root,
                                divergent_records: request.missing_paths,
                                sub_roots: HashMap::new(),
                                complete: true,
                            })),
                        }
                    }
                    _ => SyncResponse {
                        session_id,
                        seq: message.seq,
                        error: "phase is not implemented by this node".into(),
                        phase: None,
                    },
                };
                if tx.send(Ok(response)).await.is_err() {
                    break;
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn get_merkle_root(
        &self,
        request: Request<MerkleRootRequest>,
    ) -> Result<Response<MerkleRootResponse>, Status> {
        let partition_key = request.into_inner().partition_key;
        if partition_key.is_empty() {
            return Err(Status::invalid_argument("partition_key is required"));
        }
        let (merkle_root, leaf_count, tree_height) = self.merkle(&partition_key).await?;
        Ok(Response::new(MerkleRootResponse {
            partition_key,
            merkle_root,
            leaf_count,
            tree_height,
        }))
    }

    async fn get_delta(
        &self,
        request: Request<DeltaRequest>,
    ) -> Result<Response<Self::GetDeltaStream>, Status> {
        let request = request.into_inner();
        if request.partition_key.is_empty() {
            return Err(Status::invalid_argument("partition_key is required"));
        }
        let offset = request.offset as i64;
        let limit = if request.limit == 0 {
            self.config.protocol.max_batch_size as i64
        } else {
            request.limit.min(self.config.protocol.max_batch_size) as i64
        };
        let records = self
            .events
            .fetch_records(
                &request.partition_key,
                &request.record_ids,
                offset,
                limit,
                request.include_payload,
            )
            .await
            .map_err(internal)?;
        let total = self
            .events
            .count_records(&request.partition_key, &request.record_ids)
            .await
            .map_err(internal)? as u32;
        let has_more = (offset + records.len() as i64) < total as i64;
        let response = DeltaResponse {
            partition_key: request.partition_key,
            records: records.iter().map(to_proto_record).collect(),
            total_available: total,
            has_more,
            next_offset: if has_more {
                (offset + records.len() as i64) as u32
            } else {
                0
            },
        };
        let (tx, rx) = mpsc::channel(1);
        tx.send(Ok(response))
            .await
            .map_err(|_| Status::cancelled("client disconnected"))?;
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn apply_delta(
        &self,
        request: Request<tonic::Streaming<ApplyDeltaRequest>>,
    ) -> Result<Response<ApplyDeltaResponse>, Status> {
        let mut inbound = request.into_inner();
        let mut partition_key = String::new();
        let mut applied = 0u32;
        let mut skipped = 0u32;
        while let Some(item) = inbound.message().await? {
            partition_key = item.partition_key.clone();
            let Some(batch) = item.batch else {
                skipped += 1;
                continue;
            };
            let mut events = Vec::with_capacity(batch.records.len());
            for record in batch.records {
                let event_id = match uuid::Uuid::parse_str(&record.record_id) {
                    Ok(id) => id,
                    Err(_) => {
                        skipped += 1;
                        continue;
                    }
                };
                let event = SyncEvent {
                    event_id,
                    node_id: self.local_uuid(),
                    event_type: record.record_type,
                    partition_key: partition_key.clone(),
                    payload: record.payload,
                    version_vector: from_proto_version(record.version_vector),
                    local_timestamp: record
                        .local_timestamp
                        .map(|ts| timestamp_nanos(&ts))
                        .unwrap_or_else(now_nanos),
                    signature: record.signature,
                    signing_key_id: self.config.security.signing_key_id.clone(),
                    schema_version: "1.0".into(),
                    metadata: HashMap::new(),
                    created_at: record
                        .local_timestamp
                        .as_ref()
                        .and_then(timestamp_to_datetime)
                        .unwrap_or_else(chrono::Utc::now),
                };
                events.push(event);
            }
            self.events.append_batch(&events).await.map_err(internal)?;
            applied += events.len() as u32;
        }
        if !partition_key.is_empty() {
            let (merkle_root, _, _) = self.merkle(&partition_key).await?;
            self.checkpoints
                .upsert(&Checkpoint {
                    node_id: self.local_uuid(),
                    partition_key: partition_key.clone(),
                    merkle_root,
                    last_sync_at: chrono::Utc::now(),
                    synced_events: applied as u64,
                    last_error: None,
                })
                .await
                .map_err(internal)?;
        }
        Ok(Response::new(ApplyDeltaResponse {
            partition_key,
            applied,
            skipped,
            conflicts: 0,
            new_conflicts: Vec::new(),
        }))
    }

    async fn resolve_conflict(
        &self,
        request: Request<ResolveConflictRequest>,
    ) -> Result<Response<ResolveConflictResponse>, Status> {
        let request = request.into_inner();
        let conflict_id = uuid::Uuid::from_slice(&request.conflict_id)
            .map_err(|_| Status::invalid_argument("conflict_id must be a UUID"))?;
        let resolved_by =
            uuid::Uuid::from_slice(&request.resolved_by).unwrap_or_else(|_| self.local_uuid());
        self.conflicts
            .resolve(conflict_id, &request.resolution, resolved_by)
            .await
            .map_err(internal)?;
        Ok(Response::new(ResolveConflictResponse {
            success: true,
            message: "Conflict resolved".into(),
        }))
    }

    async fn get_status(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let request = request.into_inner();
        let queue_depth = self
            .queue
            .depth(self.local_uuid())
            .await
            .map_err(internal)? as u64;
        let conflicts_pending = self.conflicts.count_open().await.map_err(internal)? as u32;
        let checkpoints = self
            .checkpoints
            .list_for_node(self.local_uuid())
            .await
            .map_err(internal)?;
        let partitions = if request.include_partitions {
            checkpoints
                .into_iter()
                .map(|c| (c.partition_key.clone(), Self::checkpoint_proto(c)))
                .collect()
        } else {
            HashMap::new()
        };
        let recovery = to_proto_recovery(self.recovery.read().await.clone());
        Ok(Response::new(StatusResponse {
            node: Some(self.local_node()),
            phase: proto::SyncPhase::Completed as i32,
            active_sessions: self.sessions.read().await.len() as u32,
            queue_depth,
            events_synced_total: 0,
            bytes_synced_total: 0,
            conflicts_pending,
            uptime: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            partitions,
            pending_queue: Vec::new(),
            recovery_state: Some(recovery),
        }))
    }

    async fn get_node_info(
        &self,
        request: Request<NodeInfoRequest>,
    ) -> Result<Response<ProtoNodeIdentity>, Status> {
        let request = request.into_inner();
        let node_id =
            uuid::Uuid::from_slice(&request.node_id).unwrap_or_else(|_| self.local_uuid());
        if node_id == self.local_uuid() {
            return Ok(Response::new(self.local_node()));
        }
        let node = self
            .nodes
            .find_by_id(node_id)
            .await
            .map_err(internal)?
            .ok_or_else(|| Status::not_found("node not found"))?;
        Ok(Response::new(to_proto_node(node)))
    }

    async fn list_peers(
        &self,
        request: Request<PeersRequest>,
    ) -> Result<Response<PeersResponse>, Status> {
        let request = request.into_inner();
        let node_type = match proto::NodeType::try_from(request.filter_type).ok() {
            Some(proto::NodeType::NationalHub) => Some(NodeType::NationalHub),
            Some(proto::NodeType::RegionalRelay) => Some(NodeType::RegionalRelay),
            Some(proto::NodeType::Edge) => Some(NodeType::Edge),
            Some(proto::NodeType::Mobile) => Some(NodeType::Mobile),
            Some(proto::NodeType::DrReplica) => Some(NodeType::DrReplica),
            _ => None,
        };
        let peers = self
            .nodes
            .find_peers(&request.region, node_type)
            .await
            .map_err(internal)?;
        Ok(Response::new(PeersResponse {
            nodes: peers.into_iter().map(to_proto_node).collect(),
        }))
    }

    async fn send_heartbeat(
        &self,
        request: Request<Heartbeat>,
    ) -> Result<Response<HeartbeatAck>, Status> {
        let request = request.into_inner();
        let node_id = uuid::Uuid::from_slice(&request.node_id)
            .map_err(|_| Status::invalid_argument("node_id must be a UUID"))?;
        self.nodes
            .update_heartbeat(node_id)
            .await
            .map_err(internal)?;
        Ok(Response::new(HeartbeatAck {
            accepted: true,
            server_time: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            requires_sync: false,
            pending_sync_partitions: Vec::new(),
        }))
    }

    async fn initiate_replay(
        &self,
        request: Request<ReplayRequest>,
    ) -> Result<Response<Self::InitiateReplayStream>, Status> {
        let request = request.into_inner();
        if request.partition_key.is_empty() {
            return Err(Status::invalid_argument("partition_key is required"));
        }
        let partition_key = request.partition_key;
        let since = chrono::Utc::now() - chrono::Duration::days(1);
        let events = self
            .events
            .replay_from(
                &partition_key,
                since,
                self.config.recovery.replay_batch_size as i64,
            )
            .await
            .map_err(internal)?;
        let replay_records = events
            .iter()
            .map(|event| {
                to_proto_record(&SyncRecord {
                    record_id: event.event_id.to_string(),
                    record_type: event.event_type.clone(),
                    payload: event.payload.clone(),
                    version_vector: event.version_vector.clone(),
                    local_timestamp: chrono::DateTime::from_timestamp_nanos(
                        event.local_timestamp as i64,
                    ),
                    signature: event.signature.clone(),
                    operation: RecordOperation::Create,
                })
            })
            .collect();
        let final_root = self.merkle(&partition_key).await?.0;
        let (tx, rx) = mpsc::channel(1);
        tx.send(Ok(ReplayEvent {
            batch_seq: 0,
            events: replay_records,
            merkle_proof: Vec::new(),
            is_final: true,
            final_merkle_root: final_root,
        }))
        .await
        .map_err(|_| Status::cancelled("client disconnected"))?;
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn get_recovery_state(
        &self,
        _request: Request<RecoveryStateRequest>,
    ) -> Result<Response<RecoveryStateResponse>, Status> {
        Ok(Response::new(RecoveryStateResponse {
            state: Some(to_proto_recovery(self.recovery.read().await.clone())),
        }))
    }
}

fn to_proto_recovery(state: RecoveryStateMachine) -> proto::RecoveryState {
    let phase = match state.current_phase {
        RecoveryPhase::Detection => proto::RecoveryPhase::Detection,
        RecoveryPhase::Isolation => proto::RecoveryPhase::Isolation,
        RecoveryPhase::Reconnection => proto::RecoveryPhase::Reconnection,
        RecoveryPhase::Catchup => proto::RecoveryPhase::Catchup,
        RecoveryPhase::Normalization => proto::RecoveryPhase::Normalization,
        RecoveryPhase::Verification => proto::RecoveryPhase::Verification,
        RecoveryPhase::Complete => proto::RecoveryPhase::Complete,
        RecoveryPhase::Failed => proto::RecoveryPhase::Failed,
    } as i32;
    let last_completed_phase = state
        .last_completed_phase
        .map(|phase| match phase {
            RecoveryPhase::Detection => proto::RecoveryPhase::Detection,
            RecoveryPhase::Isolation => proto::RecoveryPhase::Isolation,
            RecoveryPhase::Reconnection => proto::RecoveryPhase::Reconnection,
            RecoveryPhase::Catchup => proto::RecoveryPhase::Catchup,
            RecoveryPhase::Normalization => proto::RecoveryPhase::Normalization,
            RecoveryPhase::Verification => proto::RecoveryPhase::Verification,
            RecoveryPhase::Complete => proto::RecoveryPhase::Complete,
            RecoveryPhase::Failed => proto::RecoveryPhase::Failed,
        } as i32)
        .unwrap_or(0);
    proto::RecoveryState {
        phase,
        started_at: Some(to_timestamp(&state.started_at)),
        region: state.region,
        node_id: state.node_id.as_bytes().to_vec(),
        autonomous_mode: state.autonomous_mode,
        pending_events: state.pending_events,
        synced_events: state.synced_events,
        conflict_count: state.conflict_count,
        last_reconnect_attempt: state.last_reconnect_attempt.as_ref().map(to_timestamp),
        reconnect_attempts: state.reconnect_attempts,
        last_error: state.last_error.unwrap_or_default(),
        last_completed_phase,
    }
}

fn to_proto_record(record: &SyncRecord) -> ProtoSyncRecord {
    ProtoSyncRecord {
        record_id: record.record_id.clone(),
        record_type: record.record_type.clone(),
        payload: record.payload.clone(),
        version_vector: Some(proto::VersionVector {
            versions: record
                .version_vector
                .versions
                .iter()
                .map(|(id, value)| (id.to_string(), *value))
                .collect(),
            local_timestamp: record.version_vector.local_timestamp,
        }),
        local_timestamp: Some(to_timestamp(&record.local_timestamp)),
        signature: record.signature.clone(),
        operation: match record.operation {
            RecordOperation::Create => proto::RecordOperation::RecordOpCreate as i32,
            RecordOperation::Update => proto::RecordOperation::RecordOpUpdate as i32,
            RecordOperation::Delete => proto::RecordOperation::RecordOpDelete as i32,
            RecordOperation::Tombstone => proto::RecordOperation::RecordOpTombstone as i32,
        },
    }
}

fn from_proto_version(version: Option<proto::VersionVector>) -> VersionVector {
    let mut versions = HashMap::new();
    if let Some(version) = version {
        for (id, value) in version.versions {
            if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
                versions.insert(uuid, value);
            }
        }
        return VersionVector {
            versions,
            local_timestamp: version.local_timestamp,
        };
    }
    VersionVector {
        versions,
        local_timestamp: now_nanos(),
    }
}

fn to_proto_node(node: NodeIdentity) -> ProtoNodeIdentity {
    ProtoNodeIdentity {
        node_id: node.node_id.as_bytes().to_vec(),
        node_type: match node.node_type {
            NodeType::NationalHub => proto::NodeType::NationalHub as i32,
            NodeType::RegionalRelay => proto::NodeType::RegionalRelay as i32,
            NodeType::Edge => proto::NodeType::Edge as i32,
            NodeType::Mobile => proto::NodeType::Mobile as i32,
            NodeType::DrReplica => proto::NodeType::DrReplica as i32,
        },
        node_name: node.node_name,
        ministry_id: node.ministry_id.to_string(),
        site_id: node.site_id.to_string(),
        region: node.region,
        certificate_serial: node.certificate_serial,
        public_key: node.public_key,
        address: node.address,
        port: node.port as u32,
        capabilities: Some(to_proto_capabilities(node.capabilities)),
        status: match node.status {
            NodeStatus::Online => proto::NodeStatus::Online as i32,
            NodeStatus::Offline => proto::NodeStatus::Offline as i32,
            NodeStatus::Suspected => proto::NodeStatus::Suspected as i32,
            NodeStatus::Recovering => proto::NodeStatus::Recovering as i32,
            NodeStatus::Quarantined => proto::NodeStatus::Quarantined as i32,
        },
        last_heartbeat: Some(to_timestamp(&node.last_heartbeat)),
    }
}

fn to_proto_capabilities(capabilities: Capabilities) -> ProtoCapabilities {
    ProtoCapabilities {
        schema_versions: capabilities.schema_versions,
        supported_entities: capabilities.supported_entities,
        max_partitions: capabilities.max_partitions,
        max_batch_size_bytes: capabilities.max_batch_size_bytes,
        supports_compression: capabilities.supports_compression,
        compression_algorithms: capabilities.compression_algorithms,
        supports_lan_mesh: capabilities.supports_lan_mesh,
    }
}

fn timestamp_nanos(timestamp: &prost_types::Timestamp) -> u64 {
    (timestamp.seconds.saturating_mul(1_000_000_000) + timestamp.nanos as i64).max(0) as u64
}
fn now_nanos() -> u64 {
    chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0).max(0) as u64
}
fn to_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: timestamp.timestamp(),
        nanos: timestamp.timestamp_subsec_nanos() as i32,
    }
}

fn timestamp_to_datetime(
    timestamp: &prost_types::Timestamp,
) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::from_timestamp(timestamp.seconds, timestamp.nanos.max(0) as u32)
}
fn internal(error: impl std::fmt::Display) -> Status {
    Status::internal(error.to_string())
}

#[allow(dead_code)]
fn _node_type(_node: &NodeTypeConfig) {}
