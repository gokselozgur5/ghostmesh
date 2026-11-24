use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024; // 64 KiB

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TransferError {
    #[error("chunk index {0} out of bounds, total {1}")]
    IndexOutOfBounds(usize, usize),
    #[error("chunk hash mismatch at index {0}")]
    ChunkHashMismatch(usize),
    #[error("manifest root hash mismatch")]
    RootHashMismatch,
    #[error("transfer incomplete: missing {0} chunks")]
    IncompleteTransfer(usize),
    #[error("empty data payload")]
    EmptyData,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub index: usize,
    pub offset: u64,
    pub length: usize,
    pub hash: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkManifest {
    pub file_name: String,
    pub total_size: u64,
    pub chunk_size: usize,
    pub root_hash: [u8; 32],
    pub chunks: Vec<ChunkInfo>,
}

impl ChunkManifest {
    pub fn build(file_name: String, data: &[u8], chunk_size: usize) -> Result<Self, TransferError> {
        if data.is_empty() {
            return Err(TransferError::EmptyData);
        }
        let chunk_size = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
        let mut chunks = Vec::new();
        let mut root_hasher = blake3::Hasher::new();

        for (i, slice) in data.chunks(chunk_size).enumerate() {
            let chunk_hash = *blake3::hash(slice).as_bytes();
            root_hasher.update(&chunk_hash);
            chunks.push(ChunkInfo {
                index: i,
                offset: (i * chunk_size) as u64,
                length: slice.len(),
                hash: chunk_hash,
            });
        }

        let root_hash = *root_hasher.finalize().as_bytes();
        Ok(Self {
            file_name,
            total_size: data.len() as u64,
            chunk_size,
            root_hash,
            chunks,
        })
    }
}
