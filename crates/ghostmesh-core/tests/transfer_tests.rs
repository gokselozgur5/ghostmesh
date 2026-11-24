use ghostmesh_core::transfer::{ChunkManifest, TransferError, TransferState};

#[test]
fn chunking_manifest_and_reassembly() {
    let raw_data = b"Ghostmesh P2P content-addressed chunking test vector with multiple chunks.";
    let chunk_size = 16;
    let manifest =
        ChunkManifest::build("test.txt".to_string(), raw_data, chunk_size).expect("valid manifest");

    assert_eq!(manifest.chunks.len(), 5);
    let mut state = TransferState::new(manifest.clone());

    // Deliver chunks out of order: 3, 1, 4, 0, 2
    let indices = [3, 1, 4, 0, 2];
    for &idx in &indices {
        let chunk_info = &manifest.chunks[idx];
        let slice = &raw_data[chunk_info.offset as usize..chunk_info.offset as usize + chunk_info.length];
        state.verify_and_store_chunk(idx, slice).expect("valid chunk");
    }

    assert!(state.is_complete());
    let reassembled = state.reassemble().expect("successful reassembly");
    assert_eq!(reassembled, raw_data);
}

#[test]
fn chunk_corruption_and_incomplete_detection() {
    let raw_data = b"Integrity verification test for corrupted chunks.";
    let chunk_size = 10;
    let manifest =
        ChunkManifest::build("doc.bin".to_string(), raw_data, chunk_size).expect("valid manifest");

    let mut state = TransferState::new(manifest.clone());

    // Incomplete transfer error
    assert_eq!(
        state.reassemble().unwrap_err(),
        TransferError::IncompleteTransfer(manifest.chunks.len())
    );

    // Corrupted chunk rejected
    let mut bad_chunk = vec![0u8; manifest.chunks[0].length];
    bad_chunk[0] = 0xff;
    assert_eq!(
        state.verify_and_store_chunk(0, &bad_chunk).unwrap_err(),
        TransferError::ChunkHashMismatch(0)
    );
}
