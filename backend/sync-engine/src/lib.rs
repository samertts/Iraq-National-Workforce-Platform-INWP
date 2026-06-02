#![allow(dead_code)]
#![allow(clippy::large_enum_variant)]

pub mod anti_corruption;
pub mod chaos;
pub mod config;
pub mod conflict_journal;
pub mod consolidation;
pub mod control_plane;
pub mod core;
pub mod edge;
pub mod error;
pub mod event_store;
pub mod events;
pub mod federation;
pub mod governance;
pub mod observability;
pub mod partition_recovery;
pub mod protocol;
pub mod reconciliation;
pub mod recovery;
pub mod runtime;
pub mod replay_verifier;
pub mod security;
pub mod security_governance;
pub mod storage;
pub mod survivability;
pub mod transport;
pub mod validation;
pub mod vector_clock;

pub use config::SyncEngineConfig;
pub use error::{SyncEngineError, SyncResult};
