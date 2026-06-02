use crate::error::{SyncEngineError, SyncResult};
use async_nats::jetstream;
use tracing::info;

pub struct NatsTransport {
    client: async_nats::Client,
    jetstream: Option<jetstream::Context>,
}

impl NatsTransport {
    pub async fn connect(url: &str) -> SyncResult<Self> {
        let client = async_nats::connect(url).await.map_err(|e| SyncEngineError::Nats(e.into()))?;
        info!(url = %url, "Connected to NATS");

        let jetstream = Some(jetstream::new(client.clone()));

        Ok(Self {
            client,
            jetstream,
        })
    }

    pub fn client(&self) -> &async_nats::Client {
        &self.client
    }

    pub fn jetstream(&self) -> Option<&jetstream::Context> {
        self.jetstream.as_ref()
    }

    pub async fn create_stream(&self, stream_name: &str, subjects: &[String]) -> SyncResult<()> {
        if let Some(js) = &self.jetstream {
            js.create_stream(jetstream::stream::Config {
                name: stream_name.to_string(),
                subjects: subjects.to_vec(),
                max_age: std::time::Duration::from_secs(30 * 24 * 60 * 60),
                storage: jetstream::stream::StorageType::File,
                ..Default::default()
            })
            .await
            .map_err(|e| SyncEngineError::Nats(e.into()))?;
            info!(stream = %stream_name, "Created NATS JetStream");
        }
        Ok(())
    }

    pub async fn close(self) {
        let _ = self.client.flush().await;
        info!("NATS connection closed");
    }
}
