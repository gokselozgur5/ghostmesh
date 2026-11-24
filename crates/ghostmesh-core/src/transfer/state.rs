use super::manifest::{ChunkManifest, TransferError};

#[derive(Clone, Debug)]
pub struct TransferState {
    pub manifest: ChunkManifest,
    received_mask: Vec<bool>,
    stored_chunks: Vec<Option<Vec<u8>>>,
}

impl TransferState {
    pub fn new(manifest: ChunkManifest) -> Self {
        let count = manifest.chunks.len();
        Self {
            manifest,
            received_mask: vec![false; count],
            stored_chunks: vec![None; count],
        }
    }

    pub fn is_chunk_received(&self, index: usize) -> bool {
        self.received_mask.get(index).copied().unwrap_or(false)
    }

    pub fn missing_chunks(&self) -> Vec<usize> {
        self.received_mask
            .iter()
            .enumerate()
            .filter_map(|(idx, &rec)| if !rec { Some(idx) } else { None })
            .collect()
    }

    pub fn is_complete(&self) -> bool {
        self.received_mask.iter().all(|&r| r)
    }

    pub fn verify_and_store_chunk(
        &mut self,
        index: usize,
        chunk_data: &[u8],
    ) -> Result<(), TransferError> {
        let expected = self
            .manifest
            .chunks
            .get(index)
            .ok_or(TransferError::IndexOutOfBounds(index, self.manifest.chunks.len()))?;

        if chunk_data.len() != expected.length {
            return Err(TransferError::ChunkHashMismatch(index));
        }

        let actual_hash = *blake3::hash(chunk_data).as_bytes();
        if actual_hash != expected.hash {
            return Err(TransferError::ChunkHashMismatch(index));
        }

        self.stored_chunks[index] = Some(chunk_data.to_vec());
        self.received_mask[index] = true;
        Ok(())
    }

    pub fn reassemble(&self) -> Result<Vec<u8>, TransferError> {
        let missing = self.missing_chunks();
        if !missing.is_empty() {
            return Err(TransferError::IncompleteTransfer(missing.len()));
        }

        let mut output = Vec::with_capacity(self.manifest.total_size as usize);
        let mut root_hasher = blake3::Hasher::new();

        for chunk_opt in &self.stored_chunks {
            let chunk = chunk_opt.as_ref().ok_or(TransferError::IncompleteTransfer(1))?;
            let h = *blake3::hash(chunk).as_bytes();
            root_hasher.update(&h);
            output.extend_from_slice(chunk);
        }

        if *root_hasher.finalize().as_bytes() != self.manifest.root_hash {
            return Err(TransferError::RootHashMismatch);
        }

        Ok(output)
    }
}
