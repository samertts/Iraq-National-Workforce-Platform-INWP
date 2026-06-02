use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyncEngineError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Storage error: {0}")]
    Storage(#[from] sqlx::Error),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("Reconciliation error: {0}")]
    Reconciliation(String),

    #[error("Recovery error: {0}")]
    Recovery(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Compression error: {0}")]
    Compression(String),

    #[error("gRPC error: {0}")]
    Grpc(Box<tonic::Status>),

    #[error("NATS error: {0}")]
    Nats(#[from] async_nats::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Partition error: {0}")]
    Partition(String),

    #[error("Sync session error: {0}")]
    SyncSession(String),

    #[error("Conflict resolution error: {0}")]
    ConflictResolution(String),

    #[error("Corruption detected: {0}")]
    Corruption(String),

    #[error("Merkle tree error: {0}")]
    Merkle(String),
}

impl From<SyncEngineError> for tonic::Status {
    fn from(err: SyncEngineError) -> Self {
        let code = match &err {
            SyncEngineError::NotFound(_) => tonic::Code::NotFound,
            SyncEngineError::AlreadyExists(_) => tonic::Code::AlreadyExists,
            SyncEngineError::Validation(_) => tonic::Code::InvalidArgument,
            SyncEngineError::Storage(_) => tonic::Code::Internal,
            SyncEngineError::Corruption(_) => tonic::Code::DataLoss,
            SyncEngineError::Security(_) => tonic::Code::PermissionDenied,
            SyncEngineError::Crypto(_) => tonic::Code::Unauthenticated,
            _ => tonic::Code::Internal,
        };
        tonic::Status::new(code, err.to_string())
    }
}

impl From<tonic::Status> for SyncEngineError {
    fn from(status: tonic::Status) -> Self {
        SyncEngineError::Grpc(Box::new(status))
    }
}

impl From<tonic::transport::Error> for SyncEngineError {
    fn from(e: tonic::transport::Error) -> Self {
        SyncEngineError::Transport(e.to_string())
    }
}

pub type SyncResult<T> = Result<T, SyncEngineError>;
