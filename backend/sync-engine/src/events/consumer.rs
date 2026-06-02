use crate::error::{SyncEngineError, SyncResult};
use crate::events::contract::SyncEvent;
use async_nats::jetstream;
use futures::{StreamExt, TryStreamExt};
use tracing::{info, debug};

pub struct EventConsumer {
    node_id: uuid::Uuid,
    subscription: Option<async_nats::Subscriber>,
    jetstream_consumer: Option<jetstream::consumer::PullConsumer>,
}

impl EventConsumer {
    pub fn new(node_id: uuid::Uuid) -> Self {
        Self {
            node_id,
            subscription: None,
            jetstream_consumer: None,
        }
    }

    pub async fn subscribe(
        &mut self,
        client: &async_nats::Client,
        subjects: &[String],
    ) -> SyncResult<()> {
        let subject = subjects.join("|");
        let sub = client.subscribe(subject).await.map_err(|e| SyncEngineError::Nats(e.into()))?;
        self.subscription = Some(sub);
        info!(subjects = ?subjects, "Subscribed to sync events");
        Ok(())
    }

    pub async fn subscribe_jetstream(
        &mut self,
        context: &jetstream::Context,
        stream: &str,
        consumer_name: &str,
    ) -> SyncResult<()> {
        let consumer = context
            .create_consumer_on_stream(
                jetstream::consumer::pull::Config {
                    name: Some(consumer_name.into()),
                    durable_name: Some(consumer_name.into()),
                    ..Default::default()
                },
                stream,
            )
            .await
            .map_err(|e| SyncEngineError::Nats(Box::new(e)))?;
        self.jetstream_consumer = Some(consumer);
        info!(stream = %stream, consumer = %consumer_name, "Subscribed to JetStream");
        Ok(())
    }

    pub async fn consume(&mut self) -> SyncResult<Option<SyncEvent>> {
        if let Some(ref mut sub) = self.subscription {
            if let Some(msg) = sub.next().await {
                let event: SyncEvent = serde_json::from_slice(&msg.payload)?;
                debug!(
                    event_id = %event.event_id,
                    event_type = %event.event_type,
                    "Event consumed"
                );
                return Ok(Some(event));
            }
        }

        if let Some(ref consumer) = self.jetstream_consumer {
                if let Ok(mut messages) = consumer.messages().await {
                if let Some(msg_result) = messages.try_next().await.map_err(|e| SyncEngineError::Nats(Box::new(e)))? {
                    let event: SyncEvent = serde_json::from_slice(&msg_result.payload)?;
                    msg_result.ack().await.map_err(SyncEngineError::Nats)?;
                    return Ok(Some(event));
                }
            }
        }

        Ok(None)
    }

    pub fn close(&mut self) {
        self.subscription = None;
        self.jetstream_consumer = None;
        info!("Event consumer closed");
    }
}
