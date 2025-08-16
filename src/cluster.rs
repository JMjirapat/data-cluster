use std::{
    collections::{BTreeMap, HashSet},
    hash::{Hash, Hasher},
};

use crate::shard::Shard;

pub trait ClusterTrait {
    fn is_empty(&self) -> bool;
    fn add_node(&mut self, node: Shard);
    fn remove_node(&mut self, node: Shard);
    fn get(&self, key: &str) -> Option<Shard>;
    fn nodes(&self) -> Vec<Shard>;
}

#[derive(Debug, Clone)]
pub struct HashRing {
    ring: BTreeMap<u64, Shard>,
    replicas: usize,
}

impl HashRing {
    pub fn new(replicas: usize) -> Self {
        HashRing {
            ring: BTreeMap::new(),
            replicas,
        }
    }
}

impl ClusterTrait for HashRing {
    fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }

    fn add_node(&mut self, node: Shard) {
        for i in 0..self.replicas {
            let node_key = format!("{}_{}", node.get_name(), i);
            let hash = hash64(&node_key);
            self.ring.insert(hash, node.clone());
        }
    }

    fn remove_node(&mut self, node: Shard) {
        for i in 0..self.replicas {
            let node_key = format!("{}_{}", node.get_name(), i);
            let hash = hash64(&node_key);
            self.ring.remove(&hash);
        }
    }

    fn get(&self, key: &str) -> Option<Shard> {
        if self.ring.is_empty() {
            return None;
        }
        let h = hash64(key);
        self.ring
            .range(h..)
            .map(|(_, n)| n.clone())
            .next()
            .or_else(|| self.ring.values().next().cloned())
    }

    fn nodes(&self) -> Vec<Shard> {
        let mut seen = HashSet::new();
        self.ring
            .values()
            .filter_map(|node| {
                if seen.insert(node.get_name()) {
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
