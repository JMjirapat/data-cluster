use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
};

pub struct HashRing<T: Clone> {
    ring: BTreeMap<u64, T>,
    replicas: usize,
}

impl<T: Clone + std::fmt::Display> HashRing<T> {
    pub fn new(replicas: usize) -> Self {
        HashRing {
            ring: BTreeMap::new(),
            replicas,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    pub fn add_node(&mut self, node: &T) {
        for i in 0..self.replicas {
            let node_key = format!("{}_{}", node.clone(), i);
            let hash = hash64(&node_key);
            self.ring.insert(hash, node.clone());
        }
    }

    pub fn remove_node(&mut self, node: T) {
        for i in 0..self.replicas {
            let node_key = format!("{}_{}", node.clone(), i);
            let hash = hash64(&node_key);
            self.ring.remove(&hash);
        }
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        if self.ring.is_empty() {
            return None;
        }
        let h = hash64(key);
        // First node with hash >= h, else wrap to first
        self.ring
            .range(h..)
            .map(|(_, n)| n)
            .next()
            .or_else(|| self.ring.values().next())
    }

    pub fn nodes(&self) -> Vec<&T> {
        self.ring.values().collect()
    }
}

fn hash64<T: Hash + ?Sized>(item: &T) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    item.hash(&mut hasher);
    hasher.finish()
}
