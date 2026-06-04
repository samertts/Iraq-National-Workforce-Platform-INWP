use crate::config::SyncEngineConfig;
use crate::error::SyncResult;
use crate::protocol::SyncSession;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

pub mod proto {
    tonic::include_proto!("inwp.sync.v1");
}

use proto::sync_service_server::{SyncService, SyncServiceServer};
use proto::{
    ApplyDeltaRequest, ApplyDeltaResponse, DeltaRequest, DeltaResponse, Heartbeat, HeartbeatAck,
    MerkleRootRequest, MerkleRootResponse, NodeIdentity, NodeInfoRequest, PeersRequest,
    PeersResponse, RecoveryStateRequest, RecoveryStateResponse, ReplayEvent, ReplayRequest,
    ResolveConflictRequest, ResolveConflictResponse, StatusRequest, StatusResponse, SyncRequest,
    SyncResponse,
};

#[derive(Clone)]
pub struct GrpcServer {
    config: SyncEngineConfig,
    pool: crate::storage::Pool,
    mesh_discovery: Arc<crate::transport::mesh::MeshDiscovery>,
    sessions: Arc<RwLock<HashMap<uuid::Uuid, SyncSession>>>,
}

impl GrpcServer {
    pub fn new(
        config: SyncEngineConfig,
        pool: crate::storage::Pool,
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

        Server::builder()
            .add_service(SyncServiceServer::new(sync_service))
            .serve(addr)
            .await?;

        Ok(())
    }
}

pub struct SyncServiceImpl {
    config: SyncEngineConfig,
    pool: crate::storage::Pool,
    sessions: Arc<RwLock<HashMap<uuid::Uuid, SyncSession>>>,
}

impl SyncServiceImpl {
    pub fn new(
        config: SyncEngineConfig,
        pool: crate::storage::Pool,
        sessions: Arc<RwLock<HashMap<uuid::Uuid, SyncSession>>>,
    ) -> Self {
        Self {
            config,
            pool,
            sessions,
        }
    }
}

#[tonic::async_trait]
impl SyncService for SyncServiceImpl {
    type InitiateSyncStream = tonic::codec::Streaming<SyncResponse>;
    type GetDeltaStream = ReceiverStream<Result<DeltaResponse, Status>>;

    async fn initiate_sync(
        &self,
        _request: Request<tonic::Streaming<SyncRequest>>,
    ) -> Result<Response<Self::InitiateSyncStream>, Status> {
        Err(Status::unimplemented("Streaming sync not yet implemented"))
    }

    async fn get_merkle_root(
        &self,
        request: Request<MerkleRootRequest>,
    ) -> Result<Response<MerkleRootResponse>, Status> {
        let req = request.into_inner();
        Ok(Response::new(MerkleRootResponse {
            partition_key: req.partition_key,
            merkle_root: vec![],
            leaf_count: 0,
            tree_height: 0,
        }))
    }

    async fn get_delta(
        &self,
        request: Request<DeltaRequest>,
    ) -> Result<Response<Self::GetDeltaStream>, Status> {
        let _req = request.into_inner();
        let (tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            let _ = tx
                .send(Ok(DeltaResponse {
                    partition_key: String::new(),
                    records: vec![],
                    total_available: 0,
                    has_more: false,
                    next_offset: 0,
                }))
                .await;
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn apply_delta(
        &self,
        _request: Request<tonic::Streaming<ApplyDeltaRequest>>,
    ) -> Result<Response<ApplyDeltaResponse>, Status> {
        Ok(Response::new(ApplyDeltaResponse {
            partition_key: String::new(),
            applied: 0,
            skipped: 0,
            conflicts: 0,
            new_conflicts: vec![],
        }))
    }

    async fn resolve_conflict(
        &self,
        request: Request<ResolveConflictRequest>,
    ) -> Result<Response<ResolveConflictResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(ResolveConflictResponse {
            success: true,
            message: "Conflict resolved".into(),
        }))
    }

    async fn get_status(
        &self,
        _request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        Ok(Response::new(StatusResponse {
            node: Some(NodeIdentity {
                node_id: self.config.node_id_bytes().to_vec(),
                node_type: 0,
                node_name: self.config.node.node_name.clone(),
                ministry_id: self.config.node.ministry_id.clone(),
                site_id: self.config.node.site_id.clone(),
                region: self.config.node.region.clone(),
                certificate_serial: String::new(),
                public_key: vec![],
                address: self.config.transport.grpc_listen.clone(),
                port: self.config.transport.grpc_port as u32,
                capabilities: None,
                status: 0,
                last_heartbeat: None,
            }),
            phase: 0,
            active_sessions: 0,
            queue_depth: 0,
            events_synced_total: 0,
            bytes_synced_total: 0,
            conflicts_pending: 0,
            uptime: None,
            partitions: HashMap::new(),
            pending_queue: vec![],
            recovery_state: None,
        }))
    }

    async fn get_node_info(
        &self,
        _request: Request<NodeInfoRequest>,
    ) -> Result<Response<NodeIdentity>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn list_peers(
        &self,
        _request: Request<PeersRequest>,
    ) -> Result<Response<PeersResponse>, Status> {
        Ok(Response::new(PeersResponse { nodes: vec![] }))
    }

    async fn send_heartbeat(
        &self,
        _request: Request<Heartbeat>,
    ) -> Result<Response<HeartbeatAck>, Status> {
        Ok(Response::new(HeartbeatAck {
            accepted: true,
            server_time: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            requires_sync: false,
            pending_sync_partitions: vec![],
        }))
    }

    type InitiateReplayStream = tonic::codec::Streaming<ReplayEvent>;

    async fn initiate_replay(
        &self,
        _request: Request<ReplayRequest>,
    ) -> Result<Response<Self::InitiateReplayStream>, Status> {
        Err(Status::unimplemented("Replay not yet implemented"))
    }

    async fn get_recovery_state(
        &self,
        _request: Request<RecoveryStateRequest>,
    ) -> Result<Response<RecoveryStateResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
}
