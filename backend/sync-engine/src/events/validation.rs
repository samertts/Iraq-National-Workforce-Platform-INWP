use crate::error::{SyncEngineError, SyncResult};
use crate::events::contract::SyncEvent;
use crate::security::signing::SigningEngine;

pub struct EventValidator {
    signing_engine: SigningEngine,
    max_event_size: u64,
    max_event_age_seconds: i64,
}

impl EventValidator {
    pub fn new(
        signing_engine: SigningEngine,
        max_event_size: u64,
        max_event_age_seconds: i64,
    ) -> Self {
        Self {
            signing_engine,
            max_event_size,
            max_event_age_seconds,
        }
    }

    pub fn validate(&self, event: &SyncEvent) -> SyncResult<()> {
        self.validate_size(event)?;
        self.validate_timestamp(event)?;
        self.validate_signature(event)?;
        self.validate_schema(event)?;
        Ok(())
    }

    fn validate_size(&self, event: &SyncEvent) -> SyncResult<()> {
        let total_size = event.payload.len() + event.metadata.len() * 64;
        if total_size as u64 > self.max_event_size {
            return Err(SyncEngineError::Validation(format!(
                "Event size {} exceeds maximum {}",
                total_size, self.max_event_size
            )));
        }
        Ok(())
    }

    fn validate_timestamp(&self, event: &SyncEvent) -> SyncResult<()> {
        let age = chrono::Utc::now() - event.created_at;
        if age.num_seconds() > self.max_event_age_seconds {
            return Err(SyncEngineError::Validation(format!(
                "Event age {}s exceeds maximum {}s",
                age.num_seconds(),
                self.max_event_age_seconds
            )));
        }
        Ok(())
    }

    fn validate_signature(&self, event: &SyncEvent) -> SyncResult<()> {
        if event.signature.is_empty() {
            return Err(SyncEngineError::Validation("Event has no signature".into()));
        }
        self.signing_engine
            .verify(&event.payload, &event.signature)
            .map_err(|_| SyncEngineError::Validation("Event signature verification failed".into()))
    }

    fn validate_schema(&self, event: &SyncEvent) -> SyncResult<()> {
        let supported_schemas = ["1.0", "1.1"];
        if !supported_schemas.contains(&event.schema_version.as_str()) {
            return Err(SyncEngineError::Validation(format!(
                "Unsupported schema version: {}",
                event.schema_version
            )));
        }
        Ok(())
    }
}
