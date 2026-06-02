use crate::error::{SyncEngineError, SyncResult};
use zstd::bulk;

#[derive(Debug, Clone)]
pub struct SyncCodec {
    pub compression_level_full: i32,
    pub compression_level_delta: i32,
    pub chunk_size: usize,
}

impl SyncCodec {
    pub fn new(compression_level_full: i32, compression_level_delta: i32, chunk_size: usize) -> Self {
        Self {
            compression_level_full,
            compression_level_delta,
            chunk_size,
        }
    }

    pub fn compress_full(&self, data: &[u8]) -> SyncResult<Vec<u8>> {
        let compressed = bulk::compress(data, self.compression_level_full)
            .map_err(|e| SyncEngineError::Compression(format!("zstd compression error: {}", e)))?;
        Ok(compressed)
    }

    pub fn compress_delta(&self, data: &[u8]) -> SyncResult<Vec<u8>> {
        let compressed = bulk::compress(data, self.compression_level_delta)
            .map_err(|e| SyncEngineError::Compression(format!("zstd compression error: {}", e)))?;
        Ok(compressed)
    }

    pub fn decompress(&self, data: &[u8]) -> SyncResult<Vec<u8>> {
        let decompressed = bulk::decompress(data, self.chunk_size * 10)
            .map_err(|e| SyncEngineError::Compression(format!("zstd decompression error: {}", e)))?;
        Ok(decompressed)
    }

    pub fn compress_with_dict(&self, data: &[u8], dict: &[u8]) -> SyncResult<Vec<u8>> {
        let mut compressor = bulk::Compressor::with_dictionary(self.compression_level_delta, dict)
            .map_err(|e| SyncEngineError::Compression(format!("zstd dict compressor error: {}", e)))?;
        compressor.compress(data)
            .map_err(|e| SyncEngineError::Compression(format!("zstd dict compression error: {}", e)))
    }

    pub fn decompress_with_dict(&self, data: &[u8], dict: &[u8]) -> SyncResult<Vec<u8>> {
        let mut decompressor = bulk::Decompressor::with_dictionary(dict)
            .map_err(|e| SyncEngineError::Compression(format!("zstd dict decompressor error: {}", e)))?;
        decompressor.decompress(data, self.chunk_size * 10)
            .map_err(|e| SyncEngineError::Compression(format!("zstd dict decompression error: {}", e)))
    }

    pub fn ratio(&self, original: usize, compressed: usize) -> f64 {
        if original == 0 {
            return 1.0;
        }
        compressed as f64 / original as f64
    }

    pub fn chunk_data(&self, data: &[u8]) -> Vec<Vec<u8>> {
        data.chunks(self.chunk_size).map(|c| c.to_vec()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let codec = SyncCodec::new(3, 3, 1024 * 1024);
        let data = b"Hello, INWP Sync Engine! This is test data for compression.";
        let compressed = codec.compress_delta(data).unwrap();
        let decompressed = codec.decompress(&compressed).unwrap();
        assert_eq!(data.to_vec(), decompressed);
    }

    #[test]
    fn test_compression_ratio_reasonable() {
        let codec = SyncCodec::new(19, 3, 1024 * 1024);
        let data = vec![b'A'; 10_000];
        let compressed = codec.compress_full(&data).unwrap();
        let ratio = codec.ratio(data.len(), compressed.len());
        assert!(ratio < 0.5, "Highly repetitive data should compress well");
    }

    #[test]
    fn test_chunk_data() {
        let codec = SyncCodec::new(3, 3, 100);
        let data = vec![0u8; 250];
        let chunks = codec.chunk_data(&data);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].len(), 100);
        assert_eq!(chunks[1].len(), 100);
        assert_eq!(chunks[2].len(), 50);
    }
}
