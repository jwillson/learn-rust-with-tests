// ANCHOR: code
pub struct LruCache<K, V> {
    capacity: usize,
    // Entries in recency order: front = least-recently-used, back = most-recent.
    entries: Vec<(K, V)>,
}

impl<K: PartialEq, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> LruCache<K, V> {
        assert!(capacity > 0, "capacity must be greater than zero");
        LruCache {
            capacity,
            entries: Vec::new(),
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let index = self.entries.iter().position(|(k, _)| k == key)?;
        // Touching a key makes it the most-recently-used, so move it to the back.
        let entry = self.entries.remove(index);
        self.entries.push(entry);
        Some(&self.entries.last().unwrap().1)
    }

    pub fn put(&mut self, key: K, value: V) {
        if let Some(index) = self.entries.iter().position(|(k, _)| *k == key) {
            self.entries.remove(index);
        } else if self.entries.len() == self.capacity {
            // Full and this is a new key: evict the least-recently-used (front).
            self.entries.remove(0);
        }
        self.entries.push((key, value));
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
// ANCHOR_END: code

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR: test
    #[test]
    fn stores_and_retrieves_values() {
        let mut cache = LruCache::new(2);

        assert_eq!(cache.get(&"a"), None);

        cache.put("a", 1);
        assert_eq!(cache.get(&"a"), Some(&1));
    }

    #[test]
    fn evicts_the_least_recently_used_entry_when_full() {
        let mut cache = LruCache::new(2);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3); // capacity 2, so "a" (oldest) is evicted

        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn getting_a_key_refreshes_its_recency() {
        let mut cache = LruCache::new(2);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.get(&"a"); // "a" is now most-recent, "b" is least-recent
        cache.put("c", 3); // so "b" is evicted, not "a"

        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn putting_an_existing_key_updates_and_refreshes_it() {
        let mut cache = LruCache::new(2);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("a", 10); // update "a"; it becomes most-recent
        cache.put("c", 3); // "b" is evicted

        assert_eq!(cache.get(&"a"), Some(&10));
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"c"), Some(&3));
    }
    // ANCHOR_END: test
}
