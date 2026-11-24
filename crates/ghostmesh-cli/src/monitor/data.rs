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
