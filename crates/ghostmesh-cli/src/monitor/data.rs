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

