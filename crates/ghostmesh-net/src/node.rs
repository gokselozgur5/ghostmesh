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

    fn start_accept_loop(&self, listener: TcpListener) {
        let identity = Arc::clone(&self.identity);
        let lww_map = Arc::clone(&self.lww_map);
        let or_set = Arc::clone(&self.or_set);
        let transfer_states = Arc::clone(&self.transfer_states);
        let completed_files = Arc::clone(&self.completed_files);
        let shutdown = Arc::clone(&self.shutdown);

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if *shutdown.lock().unwrap() {
                    break;
                }
                if let Ok(stream) = stream {
                    let identity = Arc::clone(&identity);
                    let lww = Arc::clone(&lww_map);
                    let ors = Arc::clone(&or_set);
                    let xfer = Arc::clone(&transfer_states);
                    let comp = Arc::clone(&completed_files);
                    std::thread::spawn(move || {
                        let mut rng = rand_core::OsRng;
                        if let Ok(mut enc) =
                            crate::handshake::server_handshake(stream, &identity, &mut rng)
                        {
                            let _ = Self::handle_peer_session(
                                &mut enc, lww, ors, xfer, comp,
                            );
                        }
                    });
                }
            }
        });
    }

    fn on_chunk(
        name: &str,
        idx: usize,
        data: &[u8],
        xfer: &Arc<Mutex<HashMap<String, TransferState>>>,
        comp: &Arc<Mutex<HashMap<String, Vec<u8>>>>,
    ) -> Result<(), NodeError> {
        let mut states = xfer.lock().unwrap();
        if let Some(state) = states.get_mut(name) {
            state.verify_and_store_chunk(idx, data)?;
            if state.is_complete() {
                let bytes = state.reassemble()?;
                comp.lock().unwrap().insert(name.to_string(), bytes);
            }
        }
        Ok(())
    }
