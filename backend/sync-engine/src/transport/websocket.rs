use crate::error::SyncResult;
use tokio::net::TcpListener;
use tracing::info;

pub struct WebSocketServer {
    port: u16,
}

impl WebSocketServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn serve(self) -> SyncResult<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        info!("WebSocket server listening on {}", addr);

        loop {
            let (stream, peer) = listener.accept().await?;
            tokio::spawn(async move {
                info!("TCP connection from {} (WebSocket upgrade not yet implemented)", peer);
                drop(stream);
            });
        }
    }
}
