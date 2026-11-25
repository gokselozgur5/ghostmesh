use ghostmesh_net::node::MeshNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Online,
    Syncing,
    Offline,
}

impl NodeStatus {
    pub fn badge(&self) -> (&'static str, &'static str) {
        match self {
            NodeStatus::Online => ("● ONLINE", "\x1B[92m"),
            NodeStatus::Syncing => ("◐ SYNC", "\x1B[93m"),
            NodeStatus::Offline => ("○ OFFLINE", "\x1B[90m"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonitorNode {
    pub id: String,
    pub name: String,
    pub addr: String,
    pub status: NodeStatus,
    pub crdt_version: u64,
    pub active_links: usize,
}

#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub file_name: String,
    pub total_chunks: usize,
    pub completed_chunks: usize,
    pub total_bytes: u64,
    pub root_hash_prefix: String,
}

impl TransferProgress {
    pub fn percent(&self) -> u8 {
        if self.total_chunks == 0 {
            100
        } else {
            ((self.completed_chunks as f32 / self.total_chunks as f32) * 100.0) as u8
        }
    }
}

#[derive(Debug, Clone)]
pub struct MonitorState {
    pub tick: u64,
    pub nodes: Vec<MonitorNode>,
    pub matrix: Vec<Vec<Option<u32>>>,
    pub transfers: Vec<TransferProgress>,
    pub events: Vec<String>,
}

impl MonitorState {
    pub fn new() -> Self {
        Self {
            tick: 0,
            nodes: Vec::new(),
            matrix: Vec::new(),
            transfers: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn add_event(&mut self, ev: String) {
        if self.events.len() >= 8 {
            self.events.remove(0);
        }
        self.events.push(ev);
    }

    pub fn from_live_node(node: &MeshNode, tick: u64, uptime_secs: u64) -> Self {
        let mut s = Self {
            tick,
            ..Self::new()
        };
        let peer_hex = node.peer_id().to_hex();
        let short = peer_hex.get(..4).unwrap_or(&peer_hex);
        let addr = node.local_addr();
        let crdt = node.lww_map.lock().map_or(0, |m| m.entries().len() as u64);
        let peers: Vec<String> = node
            .or_set
            .lock()
            .map_or(vec![], |o| o.elements().into_iter().collect());
        let done = node.completed_files.lock().map_or(0, |f| f.len());
        let mk = |id, name, a, links| MonitorNode {
            id,
            name,
            addr: a,
            status: NodeStatus::Online,
            crdt_version: crdt,
            active_links: links,
        };
        s.nodes.push(mk(
            peer_hex.clone(),
            format!("LOCAL-{short}"),
            addr.to_string(),
            peers.len(),
        ));
        for p in &peers {
            s.nodes.push(mk(
                p.clone(),
                format!("PEER-{}", p.get(..4).unwrap_or(p)),
                "discovered".into(),
                1,
            ));
        }
        s.matrix = vec![vec![None; s.nodes.len()]; s.nodes.len()];
        if let Ok(xfers) = node.transfer_states.lock() {
            for st in xfers.values() {
                let total = st.manifest.chunks.len();
                s.transfers.push(TransferProgress {
                    file_name: st.manifest.file_name.clone(),
                    total_chunks: total,
                    completed_chunks: total.saturating_sub(st.missing_chunks().len()),
                    total_bytes: st.manifest.total_size,
                    root_hash_prefix: blake3::Hash::from(st.manifest.root_hash).to_hex()[..8]
                        .into(),
                });
            }
        }
        s.add_event(format!(
            "[LIVE] {short} @ {addr} | up {uptime_secs}s | crdt:{crdt} | peers:{} | files:{done}",
            peers.len()
        ));
        s
    }
}

impl Default for MonitorState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn mock_initial_state() -> MonitorState {
    let mut state = MonitorState::new();
    state.nodes = vec![
        MonitorNode {
            id: "node-alpha".into(),
            name: "ALPHA-01".into(),
            addr: "127.0.0.1:9001".into(),
            status: NodeStatus::Online,
            crdt_version: 142,
            active_links: 3,
        },
        MonitorNode {
            id: "node-bravo".into(),
            name: "BRAVO-02".into(),
            addr: "127.0.0.1:9002".into(),
            status: NodeStatus::Online,
            crdt_version: 142,
            active_links: 3,
        },
        MonitorNode {
            id: "node-charlie".into(),
            name: "CHARLIE-03".into(),
            addr: "127.0.0.1:9003".into(),
            status: NodeStatus::Syncing,
            crdt_version: 139,
            active_links: 2,
        },
    ];
    state.matrix = vec![
        vec![None, Some(4), Some(12)],
        vec![Some(4), None, Some(8)],
        vec![Some(12), Some(8), None],
    ];
    state.transfers = vec![TransferProgress {
        file_name: "payload.bin".into(),
        total_chunks: 32,
        completed_chunks: 14,
        total_bytes: 2097152,
        root_hash_prefix: "e3b0c442".into(),
    }];
    state.add_event("[INIT] Mesh monitor online. 3 nodes verified via BLAKE3".into());
    state
}

pub fn advance_simulation(state: &mut MonitorState) {
    state.tick += 1;
    let mut completed_event = None;
    for t in &mut state.transfers {
        if t.completed_chunks < t.total_chunks {
            t.completed_chunks += 1;
            if t.completed_chunks == t.total_chunks {
                completed_event = Some(format!(
                    "[CHUNK] File {} assembly complete (root: {})",
                    t.file_name, t.root_hash_prefix
                ));
            }
        }
    }
    if let Some(ev) = completed_event {
        state.add_event(ev);
    }
    if state.tick.is_multiple_of(5) {
        if let Some(r) = state.matrix.get_mut(0).and_then(|row| row.get_mut(1)) {
            *r = Some(3 + ((state.tick % 4) as u32));
        }
    }
    if state.tick == 18 {
        if let Some(c) = state.nodes.get_mut(2) {
            c.status = NodeStatus::Online;
            c.crdt_version = 142;
            c.active_links = 3;
            state.add_event("[SYNC] CHARLIE-03 reached CRDT LWW convergence".into());
        }
    }
}
