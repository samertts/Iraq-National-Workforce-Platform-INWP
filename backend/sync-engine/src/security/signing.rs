use crate::error::{SyncEngineError, SyncResult};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use std::path::Path;
use std::sync::Mutex;

#[derive(Clone)]
pub struct SigningEngine {
    signing_key: std::sync::Arc<Mutex<SigningKey>>,
    verifying_key: VerifyingKey,
    key_id: String,
}

impl SigningEngine {
    pub fn from_bytes(private_key: &[u8], key_id: impl Into<String>) -> SyncResult<Self> {
        let bytes: [u8; 32] = private_key.try_into().map_err(|_| {
            SyncEngineError::Crypto("Invalid private key length: expected 32 bytes".into())
        })?;

        let signing_key = SigningKey::from_bytes(&bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key: std::sync::Arc::new(Mutex::new(signing_key)),
            verifying_key,
            key_id: key_id.into(),
        })
    }

    pub fn from_file(path: impl AsRef<Path>, key_id: impl Into<String>) -> SyncResult<Self> {
        let bytes = std::fs::read(path.as_ref()).map_err(|e| {
            SyncEngineError::Crypto(format!("Failed to read signing key file: {}", e))
        })?;
        Self::from_bytes(&bytes, key_id)
    }

    pub fn generate(key_id: impl Into<String>) -> Self {
        let mut csprng = rand::rngs::OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key: std::sync::Arc::new(Mutex::new(signing_key)),
            verifying_key,
            key_id: key_id.into(),
        }
    }

    pub fn sign(&self, data: &[u8]) -> SyncResult<Vec<u8>> {
        let key = self
            .signing_key
            .lock()
            .map_err(|e| SyncEngineError::Crypto(format!("Signing key lock error: {}", e)))?;
        let signature = key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    pub fn verify(&self, data: &[u8], signature_bytes: &[u8]) -> SyncResult<()> {
        let signature = Signature::from_slice(signature_bytes)
            .map_err(|e| SyncEngineError::Crypto(format!("Invalid signature bytes: {}", e)))?;

        self.verifying_key
            .verify(data, &signature)
            .map_err(|e| SyncEngineError::Crypto(format!("Signature verification failed: {}", e)))
    }

    pub fn verify_with_key(
        data: &[u8],
        signature_bytes: &[u8],
        public_key: &[u8],
    ) -> SyncResult<()> {
        let verifying_key = VerifyingKey::from_bytes(
            public_key
                .try_into()
                .map_err(|_| SyncEngineError::Crypto("Invalid public key length".into()))?,
        )
        .map_err(|e| SyncEngineError::Crypto(format!("Invalid public key: {}", e)))?;

        let signature = Signature::from_slice(signature_bytes)
            .map_err(|e| SyncEngineError::Crypto(format!("Invalid signature bytes: {}", e)))?;

        verifying_key
            .verify(data, &signature)
            .map_err(|e| SyncEngineError::Crypto(format!("Signature verification failed: {}", e)))
    }

    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.verifying_key.to_bytes().to_vec()
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify() {
        let engine = SigningEngine::generate("test-key");
        let data = b"test data to sign";
        let signature = engine.sign(data).unwrap();
        assert!(engine.verify(data, &signature).is_ok());
    }

    #[test]
    fn test_verify_wrong_data() {
        let engine = SigningEngine::generate("test-key");
        let data = b"test data";
        let signature = engine.sign(data).unwrap();
        assert!(engine.verify(b"wrong data", &signature).is_err());
    }

    #[test]
    fn test_public_key_roundtrip() {
        let engine = SigningEngine::generate("test-key");
        let pub_key = engine.public_key_bytes();
        assert_eq!(pub_key.len(), 32);
    }
}
