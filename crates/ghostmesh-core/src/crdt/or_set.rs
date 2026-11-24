use crate::identity::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tag {
    pub peer_id: PeerId,
    pub seq: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ElementRecord<T: Ord + Clone> {
    pub element: T,
    pub tag: Tag,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrSet<T: Ord + Clone> {
    adds: BTreeSet<ElementRecord<T>>,
    removals: BTreeSet<Tag>,
    next_seq: u64,
    peer_id: PeerId,
}

impl<T: Ord + Clone> OrSet<T> {
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            adds: BTreeSet::new(),
            removals: BTreeSet::new(),
            next_seq: 0,
            peer_id,
        }
    }

    pub fn insert(&mut self, element: T) -> Tag {
        self.next_seq = self.next_seq.saturating_add(1);
        let tag = Tag {
            peer_id: self.peer_id,
            seq: self.next_seq,
        };
        self.adds.insert(ElementRecord {
            element,
            tag: tag.clone(),
        });
        tag
    }

    pub fn remove(&mut self, element: &T) {
        for record in &self.adds {
            if &record.element == element {
                self.removals.insert(record.tag.clone());
            }
        }
    }

    pub fn contains(&self, element: &T) -> bool {
        self.adds
            .iter()
            .any(|r| &r.element == element && !self.removals.contains(&r.tag))
    }

    pub fn elements(&self) -> BTreeSet<T> {
        self.adds
            .iter()
            .filter(|r| !self.removals.contains(&r.tag))
            .map(|r| r.element.clone())
            .collect()
    }

    pub fn merge(&mut self, other: Self) {
        for record in &other.adds {
            if record.tag.peer_id == self.peer_id && record.tag.seq > self.next_seq {
                self.next_seq = record.tag.seq;
            }
        }
        for tag in &other.removals {
            if tag.peer_id == self.peer_id && tag.seq > self.next_seq {
                self.next_seq = tag.seq;
            }
        }
        self.adds.extend(other.adds);
        self.removals.extend(other.removals);
    }
}
