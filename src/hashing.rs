use std::{
    collections::{BTreeMap, HashSet},
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::storage::StorageTrait;

#[derive(Debug, Clone)]
pub struct HashRing<T> {
    ring: BTreeMap<u64, Arc<T>>,
    replicas: usize,
}

impl<T: StorageTrait> HashRing<T> {
    pub fn new(replicas: usize) -> Self {
        HashRing {
            ring: BTreeMap::new(),
            replicas,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    pub fn add_node(&mut self, node: Arc<T>) {
        for i in 0..self.replicas {
            let node_key = format!("{}_{}", node.get_name(), i);
            let hash = hash64(&node_key);
            self.ring.insert(hash, node.clone());
        }
    }

    pub fn remove_node(&mut self, node: Arc<T>) {
        for i in 0..self.replicas {
            let node_key = format!("{}_{}", node.get_name(), i);
            let hash = hash64(&node_key);
            self.ring.remove(&hash);
        }
    }

    pub fn get(&self, key: &str) -> Option<Arc<T>> {
        if self.ring.is_empty() {
            return None;
        }
        let h = hash64(key);
        // First node with hash >= h, else wrap to first
        self.ring
            .range(h..)
            .map(|(_, n)| n.clone())
            .next()
            .or_else(|| self.ring.values().next().cloned())
    }

    pub fn nodes(&self) -> Vec<Arc<T>> {
        // self.ring.values().cloned().collect()
        let mut seen = HashSet::new();
        self.ring
            .values()
            .filter_map(|node| {
                if seen.insert(node.get_name().clone()) {
                    Some(node.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

fn hash64<T: Hash + ?Sized>(item: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    item.hash(&mut hasher);
    hasher.finish()
}
