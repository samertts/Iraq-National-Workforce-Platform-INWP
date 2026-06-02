use crate::error::{SyncEngineError, SyncResult};
use crate::config::SecurityConfig;
use std::sync::Arc;
use tokio_rustls::rustls::{self};

pub struct TlsConfig {
    pub server_config: Arc<rustls::ServerConfig>,
    pub client_config: Arc<rustls::ClientConfig>,
}

impl TlsConfig {
    pub fn from_files(config: &SecurityConfig) -> SyncResult<Self> {
        let cert_pem = std::fs::read_to_string(&config.tls_cert_path)
            .map_err(|e| SyncEngineError::Crypto(format!("Failed to read cert: {}", e)))?;
        let key_pem = std::fs::read_to_string(&config.tls_key_path)
            .map_err(|e| SyncEngineError::Crypto(format!("Failed to read key: {}", e)))?;
        let ca_pem = std::fs::read_to_string(&config.tls_ca_path)
            .map_err(|e| SyncEngineError::Crypto(format!("Failed to read CA: {}", e)))?;

        Self::from_pem(&cert_pem, &key_pem, &ca_pem, config.mtls_required)
    }

    pub fn from_pem(
        cert_pem: &str,
        key_pem: &str,
        ca_pem: &str,
        require_mtls: bool,
    ) -> SyncResult<Self> {
        let certs = rustls_pemfile::certs(&mut cert_pem.as_bytes())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| SyncEngineError::Crypto(format!("Failed to parse cert PEM: {}", e)))?;

        let key = rustls_pemfile::private_key(&mut key_pem.as_bytes())
            .map_err(|e| SyncEngineError::Crypto(format!("Failed to parse key PEM: {}", e)))?
            .ok_or_else(|| SyncEngineError::Crypto("No private key found in PEM".into()))?;

        let ca_certs = rustls_pemfile::certs(&mut ca_pem.as_bytes())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| SyncEngineError::Crypto(format!("Failed to parse CA PEM: {}", e)))?;

        let mut root_store = rustls::RootCertStore::empty();
        for ca_cert in &ca_certs {
            root_store.add(ca_cert.clone())
                .map_err(|e| SyncEngineError::Crypto(format!("Failed to add CA cert: {}", e)))?;
        }

        let mut server_config = rustls::ServerConfig::builder()
            .with_client_cert_verifier(
                if require_mtls {
                    rustls::server::WebPkiClientVerifier::builder(root_store.clone().into())
                        .build()
                        .map_err(|e| SyncEngineError::Crypto(format!("MTLS config error: {}", e)))?
                } else {
                    rustls::server::WebPkiClientVerifier::builder(
                        rustls::RootCertStore::empty().into()
                    )
                    .allow_unauthenticated()
                    .build()
                    .map_err(|e| SyncEngineError::Crypto(format!("TLS config error: {}", e)))?
                }
            )
            .with_single_cert(certs.clone(), key.clone_key())
            .map_err(|e| SyncEngineError::Crypto(format!("Server cert error: {}", e)))?;

        server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        let client_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self {
            server_config: Arc::new(server_config),
            client_config: Arc::new(client_config),
        })
    }
}
