use crate::handshake::HandshakeError;
use ghostmesh_core::crdt::LwwMap;
use ghostmesh_core::identity::PeerId;
use ghostmesh_core::transfer::{ChunkManifest, TransferError};
use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;
use thiserror::Error;

pub const MAX_TRANSFER_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_ACTIVE_TRANSFERS: usize = 64;
pub const MAX_CONCURRENT_PEERS: usize = 64;
pub const IO_TIMEOUT: Duration = Duration::from_secs(5);

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
    #[error("peer capacity reached")]
    PeerCapacityReached,
    #[error("active transfer limit reached")]
    TransferCapacityReached,
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
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

struct PeerSlotGuard(Arc<AtomicUsize>);

impl Drop for PeerSlotGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

pub struct MeshNode {
    pub identity: Arc<PeerIdentity>,
    pub local_addr: SocketAddr,
    pub lww_map: Arc<Mutex<LwwMap<String, String>>>,
    pub or_set: Arc<Mutex<OrSet<String>>>,
    pub transfer_states: Arc<Mutex<HashMap<String, TransferState>>>,
    pub completed_files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    active_peers: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
    listener_handle: Mutex<Option<JoinHandle<()>>>,
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
                            let _ = Self::handle_peer_session(&mut enc, lww, ors, xfer, comp);
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
            if state.is_chunk_received(idx) {
                return Ok(());
            }
            state.verify_and_store_chunk(idx, data)?;
            if state.is_complete() {
                let bytes = state.reassemble()?;
                comp.lock().unwrap().insert(name.to_string(), bytes);
                states.remove(name);
            }
        }
        Ok(())
    }

    fn handle_peer_session(
        enc: &mut crate::handshake::EncryptedStream,
        lww_map: Arc<Mutex<LwwMap<String, String>>>,
        _or_set: Arc<Mutex<OrSet<String>>>,
        transfer_states: Arc<Mutex<HashMap<String, TransferState>>>,
        completed_files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    ) -> Result<(), NodeError> {
        while let Ok(plaintext) = enc.recv_encrypted() {
            let msg: NodeMessage = serde_json::from_slice(&plaintext)?;
            match msg {
                NodeMessage::SyncLwwRequest => {
                    let delta = lww_map.lock().unwrap().delta_since(None);
                    let resp = NodeMessage::SyncLwwResponse { delta };
                    enc.send_encrypted(&serde_json::to_vec(&resp)?)?;
                }
                NodeMessage::SyncLwwResponse { delta } => {
                    lww_map.lock().unwrap().merge(delta);
                    enc.send_encrypted(&serde_json::to_vec(&NodeMessage::Ack)?)?;
                }
                NodeMessage::SendFileManifest { manifest } => {
                    manifest.validate(MAX_TRANSFER_BYTES)?;
                    let name = manifest.file_name.clone();
                    if manifest.total_size == 0 {
                        completed_files.lock().unwrap().insert(name, Vec::new());
                        enc.send_encrypted(&serde_json::to_vec(&NodeMessage::Ack)?)?;
                    } else {
                        let mut states = transfer_states.lock().unwrap();
                        if states.len() >= MAX_ACTIVE_TRANSFERS && !states.contains_key(&name) {
                            return Err(NodeError::TransferCapacityReached);
                        }
                        states.insert(name, TransferState::new(manifest)?);
                        enc.send_encrypted(&serde_json::to_vec(&NodeMessage::Ack)?)?;
                    }
                }
                NodeMessage::SendFileChunk {
                    file_name,
                    chunk_index,
                    data,
                } => {
                    Self::on_chunk(
                        &file_name,
                        chunk_index,
                        &data,
                        &transfer_states,
                        &completed_files,
                    )?;
                    enc.send_encrypted(&serde_json::to_vec(&NodeMessage::Ack)?)?;
                }
                NodeMessage::TransferComplete { .. } | NodeMessage::Ack => {}
            }
        }
        Ok(())
    }

    pub fn sync_lww_with(&self, peer_addr: SocketAddr) -> Result<(), NodeError> {
        let stream = TcpStream::connect_timeout(&peer_addr, IO_TIMEOUT)?;
        stream.set_read_timeout(Some(IO_TIMEOUT))?;
        stream.set_write_timeout(Some(IO_TIMEOUT))?;
        let mut rng = rand_core::OsRng;
        let mut enc = crate::handshake::client_handshake(stream, &self.identity, &mut rng)?;

        let req = NodeMessage::SyncLwwRequest;
        enc.send_encrypted(&serde_json::to_vec(&req)?)?;
        let resp_bytes = enc.recv_encrypted()?;
        let resp: NodeMessage = serde_json::from_slice(&resp_bytes)?;

        if let NodeMessage::SyncLwwResponse { delta } = resp {
            self.lww_map.lock().unwrap().merge(delta);
        } else {
            return Err(NodeError::Protocol("expected SyncLwwResponse".into()));
        }

        let my_delta = self.lww_map.lock().unwrap().delta_since(None);
        let push_msg = NodeMessage::SyncLwwResponse { delta: my_delta };
        enc.send_encrypted(&serde_json::to_vec(&push_msg)?)?;
        let _ = enc.recv_encrypted()?;
        Ok(())
    }

    pub fn send_file_to(
        &self,
        peer_addr: SocketAddr,
        file_name: &str,
        bytes: &[u8],
    ) -> Result<(), NodeError> {
        let manifest = ChunkManifest::build(file_name.to_string(), bytes, 64 * 1024)?;
        let stream = TcpStream::connect_timeout(&peer_addr, IO_TIMEOUT)?;
        stream.set_read_timeout(Some(IO_TIMEOUT))?;
        stream.set_write_timeout(Some(IO_TIMEOUT))?;
        let mut rng = rand_core::OsRng;
        let mut enc = crate::handshake::client_handshake(stream, &self.identity, &mut rng)?;

        let manifest_msg = NodeMessage::SendFileManifest {
            manifest: manifest.clone(),
        };
        enc.send_encrypted(&serde_json::to_vec(&manifest_msg)?)?;
        let _ = enc.recv_encrypted()?;

        for chunk_info in &manifest.chunks {
            let start = chunk_info.offset as usize;
            let end = start + chunk_info.length;
            let chunk_data = bytes[start..end].to_vec();
            let chunk_msg = NodeMessage::SendFileChunk {
                file_name: file_name.to_string(),
                chunk_index: chunk_info.index,
                data: chunk_data,
            };
            enc.send_encrypted(&serde_json::to_vec(&chunk_msg)?)?;
            let _ = enc.recv_encrypted()?;
        }
        Ok(())
    }
}

impl Drop for MeshNode {
    fn drop(&mut self) {
        self.shutdown();
    }
}
