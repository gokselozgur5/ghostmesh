use super::hlc::HlcTimestamp;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LwwEntry<V> {
    pub value: Option<V>,
    pub timestamp: HlcTimestamp,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LwwMap<K: Ord + Clone, V: Clone> {
    entries: BTreeMap<K, LwwEntry<V>>,
}

impl<K: Ord + Clone, V: Clone> LwwMap<K, V> {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn set(&mut self, key: K, value: V, timestamp: HlcTimestamp) {
        if let Some(existing) = self.entries.get_mut(&key) {
            if timestamp > existing.timestamp {
                existing.value = Some(value);
                existing.timestamp = timestamp;
            }
        } else {
            self.entries.insert(
                key,
                LwwEntry {
                    value: Some(value),
                    timestamp,
                },
            );
        }
    }

    pub fn delete(&mut self, key: K, timestamp: HlcTimestamp) {
        if let Some(existing) = self.entries.get_mut(&key) {
            if timestamp > existing.timestamp {
                existing.value = None;
                existing.timestamp = timestamp;
            }
        } else {
            self.entries.insert(
                key,
                LwwEntry {
                    value: None,
                    timestamp,
                },
            );
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).and_then(|e| e.value.as_ref())
    }

    pub fn get_entry(&self, key: &K) -> Option<&LwwEntry<V>> {
        self.entries.get(key)
    }

    pub fn entries(&self) -> &BTreeMap<K, LwwEntry<V>> {
        &self.entries
    }

    pub fn merge(&mut self, other: Self) {
        for (key, remote_entry) in other.entries {
            if let Some(local_entry) = self.entries.get_mut(&key) {
                if remote_entry.timestamp > local_entry.timestamp {
                    *local_entry = remote_entry;
                }
            } else {
                self.entries.insert(key, remote_entry);
            }
        }
    }

    pub fn delta_since(&self, threshold: Option<&HlcTimestamp>) -> Self {
        let mut delta_entries = BTreeMap::new();
        for (k, entry) in &self.entries {
            if match threshold {
                Some(thresh) => entry.timestamp > *thresh,
                None => true,
            } {
                delta_entries.insert(k.clone(), entry.clone());
            }
        }
        Self {
            entries: delta_entries,
        }
    }
}
