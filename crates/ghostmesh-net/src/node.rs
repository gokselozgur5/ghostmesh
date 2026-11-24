use crate::handshake::HandshakeError;
use ghostmesh_core::crdt::LwwMap;
use ghostmesh_core::identity::PeerId;
use ghostmesh_core::transfer::{ChunkManifest, TransferError};
use serde::{Deserialize, Serialize};
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("handshake error: {0}")]
    Handshake(#[from] HandshakeError),
    #[error("wire error: {0}")]
    Wire(#[from] crate::wire::WireError),
    #[error("transfer error: {0}")]
    Transfer(#[from] TransferError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("protocol violation: {0}")]
    Protocol(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeMessage {
    SyncLwwRequest,
    SyncLwwResponse {
        delta: LwwMap<String, String>,
    },
    SendFileManifest {
        manifest: ChunkManifest,
    },
    SendFileChunk {
        file_name: String,
        chunk_index: usize,
        data: Vec<u8>,
    },
    TransferComplete {
        file_name: String,
        root_hash: [u8; 32],
    },
    Ack,
}

use ghostmesh_core::crdt::OrSet;
use ghostmesh_core::identity::PeerIdentity;
use ghostmesh_core::transfer::TransferState;
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex};

pub struct MeshNode {
    pub identity: Arc<PeerIdentity>,
    pub local_addr: SocketAddr,
    pub lww_map: Arc<Mutex<LwwMap<String, String>>>,
    pub or_set: Arc<Mutex<OrSet<String>>>,
    pub transfer_states: Arc<Mutex<HashMap<String, TransferState>>>,
    pub completed_files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    shutdown: Arc<Mutex<bool>>,
}


impl MeshNode {
    pub fn bind(identity: PeerIdentity, bind_addr: &str) -> Result<Self, NodeError> {
        let listener = TcpListener::bind(bind_addr)?;
        let local_addr = listener.local_addr()?;
        let peer_id = identity.peer_id();
        let identity = Arc::new(identity);
        let lww_map = Arc::new(Mutex::new(LwwMap::new()));
        let or_set = Arc::new(Mutex::new(OrSet::new(peer_id)));
        let transfer_states = Arc::new(Mutex::new(HashMap::new()));
        let completed_files = Arc::new(Mutex::new(HashMap::new()));
        let shutdown = Arc::new(Mutex::new(false));

        let node = Self {
            identity,
            local_addr,
            lww_map,
            or_set,
            transfer_states,
            completed_files,
            shutdown,
        };
        node.start_accept_loop(listener);
        Ok(node)
    }

    pub fn peer_id(&self) -> PeerId {
        self.identity.peer_id()
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}
