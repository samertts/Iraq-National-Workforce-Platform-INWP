use crate::error::{SyncEngineError, SyncResult};
use crate::events::contract::SyncEvent;
use crate::security::signing::SigningEngine;
use tracing::debug;

#[derive(Clone)]
pub struct EventProducer {
    node_id: uuid::Uuid,
    signing_key_id: String,
    signing_engine: SigningEngine,
}

impl EventProducer {
    pub fn new(node_id: uuid::Uuid, signing_key_id: impl Into<String>, signing_engine: SigningEngine) -> Self {
        Self {
            node_id,
            signing_key_id: signing_key_id.into(),
            signing_engine,
        }
    }

    pub async fn produce_event(
        &self,
        event_type: impl Into<String>,
        partition_key: impl Into<String>,
        payload: Vec<u8>,
    ) -> SyncResult<SyncEvent> {
        let mut event = SyncEvent::new(
            self.node_id,
            event_type,
            partition_key,
            payload,
            &self.signing_key_id,
        );

        let signature = self.signing_engine.sign(event.payload.as_slice())?;
        event.signature = signature;

        debug!(
            event_id = %event.event_id,
            event_type = %event.event_type,
            partition = %event.partition_key,
            "Event produced"
        );

        Ok(event)
    }

    pub async fn produce_and_publish(
        &self,
        event_type: impl Into<String>,
        partition_key: impl Into<String>,
        payload: Vec<u8>,
        publisher: &impl EventPublisher,
    ) -> SyncResult<()> {
        let event = self.produce_event(event_type, partition_key, payload).await?;
        publisher.publish(&event).await
    }
}

#[async_trait::async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: &SyncEvent) -> SyncResult<()>;
    async fn publish_batch(&self, events: &[SyncEvent]) -> SyncResult<()>;
}

pub struct NatsEventPublisher {
    client: async_nats::Client,
    subject_prefix: String,
}

impl NatsEventPublisher {
    pub fn new(client: async_nats::Client, subject_prefix: impl Into<String>) -> Self {
        Self {
            client,
            subject_prefix: subject_prefix.into(),
        }
    }
}

#[async_trait::async_trait]
impl EventPublisher for NatsEventPublisher {
    async fn publish(&self, event: &SyncEvent) -> SyncResult<()> {
        let subject = format!("{}.{}", self.subject_prefix, event.event_type);
        let payload = serde_json::to_vec(event)?;
        self.client.publish(subject, payload.into()).await.map_err(|e| SyncEngineError::Nats(e.into()))?;
        Ok(())
    }

    async fn publish_batch(&self, events: &[SyncEvent]) -> SyncResult<()> {
        for event in events {
            self.publish(event).await?;
        }
        Ok(())
    }
}
