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

