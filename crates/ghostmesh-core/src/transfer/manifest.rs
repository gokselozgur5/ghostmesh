use serde::{Deserialize, Serialize};
use std::path::{Component, Path};
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
    #[error("invalid file name in manifest: {0}")]
    InvalidFileName(String),
    #[error("manifest total size {0} exceeds allowed maximum {1}")]
    SizeExceedsLimit(u64, u64),
    #[error("corrupt or non-contiguous chunk list in manifest")]
    CorruptChunkList,
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
        let chunk_size = if chunk_size == 0 {
            DEFAULT_CHUNK_SIZE
        } else {
            chunk_size
        };

        if data.is_empty() {
            let root_hash = *blake3::hash(b"").as_bytes();
            let manifest = Self {
                file_name,
                total_size: 0,
                chunk_size,
                root_hash,
                chunks: Vec::new(),
            };
            manifest.validate(u64::MAX)?;
            return Ok(manifest);
        }

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
        let manifest = Self {
            file_name,
            total_size: data.len() as u64,
            chunk_size,
            root_hash,
            chunks,
        };
        manifest.validate(u64::MAX)?;
        Ok(manifest)
    }

    pub fn validate(&self, max_total_bytes: u64) -> Result<(), TransferError> {
        if self.file_name.is_empty() || self.file_name.contains('\0') {
            return Err(TransferError::InvalidFileName(self.file_name.clone()));
        }
        let mut components = Path::new(&self.file_name).components();
        match (components.next(), components.next()) {
            (Some(Component::Normal(_)), None) => {}
            _ => return Err(TransferError::InvalidFileName(self.file_name.clone())),
        }

        if self.total_size > max_total_bytes {
            return Err(TransferError::SizeExceedsLimit(
                self.total_size,
                max_total_bytes,
            ));
        }

        if self.total_size == 0 {
            if !self.chunks.is_empty() {
                return Err(TransferError::CorruptChunkList);
            }
            if self.root_hash != *blake3::hash(b"").as_bytes() {
                return Err(TransferError::RootHashMismatch);
            }
            return Ok(());
        }

        let mut current_offset: u64 = 0;
        for (i, chunk) in self.chunks.iter().enumerate() {
            if chunk.index != i || chunk.offset != current_offset || chunk.length == 0 {
                return Err(TransferError::CorruptChunkList);
            }
            current_offset = current_offset
                .checked_add(chunk.length as u64)
                .ok_or(TransferError::CorruptChunkList)?;
        }

        if current_offset != self.total_size {
            return Err(TransferError::CorruptChunkList);
        }

        Ok(())
    }
}
