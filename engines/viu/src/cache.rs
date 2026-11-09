// SPDX-License-Identifier: MIT OR Apache-2.0
//! Small LRU shared by decoded assets and icon resolutions.
use std::collections::BTreeMap;

#[derive(Debug)]
pub(crate) struct Cache<K, V> {
    entries: BTreeMap<K, (u64, V)>,
    clock: u64,
}

impl<K: Ord + Clone, V> Default for Cache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Ord + Clone, V> Cache<K, V> {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            clock: 0,
        }
    }
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.clock += 1;
        let (age, value) = self.entries.get_mut(key)?;
        *age = self.clock;
        Some(value)
    }
    pub fn insert(&mut self, key: K, value: V) {
        if self.entries.len() >= 512 && !self.entries.contains_key(&key) {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, (age, _))| age)
                .map(|(key, _)| key.clone());
            if let Some(oldest) = oldest {
                self.entries.remove(&oldest);
            }
        }
        self.clock += 1;
        self.entries.insert(key, (self.clock, value));
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    #[cfg(test)]
    pub fn contains_key(&self, key: &K) -> bool {
        self.entries.contains_key(key)
    }
    #[cfg(test)]
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.values().map(|(_, value)| value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pressure_evicts_one_old_entry_and_keeps_recent_hits() {
        let mut cache = Cache::new();
        for i in 0..512 {
            cache.insert(i, i);
        }
        assert_eq!(cache.get(&0), Some(&0));
        cache.insert(512, 512);
        assert_eq!(cache.len(), 512);
        assert!(cache.contains_key(&0));
        assert!(!cache.contains_key(&1));
        assert!(cache.contains_key(&511));
    }
}
