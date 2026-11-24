use crate::identity::PeerId;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HlcTimestamp {
    pub physical_ms: u64,
    pub logical: u16,
    pub peer_id: PeerId,
}

impl HlcTimestamp {
    pub fn new(physical_ms: u64, logical: u16, peer_id: PeerId) -> Self {
        Self {
            physical_ms,
            logical,
            peer_id,
        }
    }

    pub fn tick(&self, now_ms: u64, peer_id: PeerId) -> Self {
        if now_ms > self.physical_ms {
            Self {
                physical_ms: now_ms,
                logical: 0,
                peer_id,
            }
        } else {
            let (logical, overflow) = self.logical.overflowing_add(1);
            let physical_ms = if overflow {
                self.physical_ms.saturating_add(1)
            } else {
                self.physical_ms
            };
            Self {
                physical_ms,
                logical,
                peer_id,
            }
        }
    }

    pub fn merge(&self, remote: &Self, now_ms: u64, peer_id: PeerId) -> Self {
        let max_phys = self.physical_ms.max(remote.physical_ms).max(now_ms);
        let base_logical = if max_phys == self.physical_ms && max_phys == remote.physical_ms {
            self.logical.max(remote.logical)
        } else if max_phys == self.physical_ms {
            self.logical
        } else if max_phys == remote.physical_ms {
            remote.logical
        } else {
            return Self {
                physical_ms: max_phys,
                logical: 0,
                peer_id,
            };
        };

        let (logical, overflow) = base_logical.overflowing_add(1);
        let physical_ms = if overflow {
            max_phys.saturating_add(1)
        } else {
            max_phys
        };

        Self {
            physical_ms,
            logical,
            peer_id,
        }
    }
}

impl Ord for HlcTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.physical_ms
            .cmp(&other.physical_ms)
            .then_with(|| self.logical.cmp(&other.logical))
            .then_with(|| self.peer_id.0.cmp(&other.peer_id.0))
    }
}

impl PartialOrd for HlcTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
