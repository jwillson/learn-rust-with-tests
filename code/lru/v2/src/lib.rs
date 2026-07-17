use std::collections::HashMap;
use std::hash::Hash;

// ANCHOR: node
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

pub struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, usize>,
    nodes: Vec<Node<K, V>>,
    head: Option<usize>, // most-recently-used
    tail: Option<usize>, // least-recently-used
}
// ANCHOR_END: node

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    pub fn new(capacity: usize) -> LruCache<K, V> {
        assert!(capacity > 0, "capacity must be greater than zero");
        LruCache {
            capacity,
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            head: None,
            tail: None,
        }
    }

    // ANCHOR: get
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let index = *self.map.get(key)?;
        self.move_to_front(index);
        Some(&self.nodes[index].value)
    }
    // ANCHOR_END: get

    // ANCHOR: put
    pub fn put(&mut self, key: K, value: V) {
        if let Some(&index) = self.map.get(&key) {
            self.nodes[index].value = value;
            self.move_to_front(index);
            return;
        }

        let index = if self.map.len() == self.capacity {
            // Full: reuse the least-recently-used slot.
            let evicted = self.tail.expect("a full cache has a tail");
            self.unlink(evicted);
            self.map.remove(&self.nodes[evicted].key);
            self.nodes[evicted] = Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
            };
            evicted
        } else {
            // Room to grow: take a fresh slot.
            self.nodes.push(Node {
                key: key.clone(),
                value,
                prev: None,
                next: None,
            });
            self.nodes.len() - 1
        };

        self.map.insert(key, index);
        self.push_front(index);
    }
    // ANCHOR_END: put

    // ANCHOR: links
    /// Detach node `i` from the list, mending its neighbours.
    fn unlink(&mut self, i: usize) {
        let prev = self.nodes[i].prev;
        let next = self.nodes[i].next;

        match prev {
            Some(p) => self.nodes[p].next = next,
            None => self.head = next,
        }
        match next {
            Some(n) => self.nodes[n].prev = prev,
            None => self.tail = prev,
        }
    }

    /// Make node `i` the head (most-recently-used).
    fn push_front(&mut self, i: usize) {
        self.nodes[i].prev = None;
        self.nodes[i].next = self.head;

        if let Some(old_head) = self.head {
            self.nodes[old_head].prev = Some(i);
        }
        self.head = Some(i);
        if self.tail.is_none() {
            self.tail = Some(i);
        }
    }

    fn move_to_front(&mut self, i: usize) {
        if self.head == Some(i) {
            return;
        }
        self.unlink(i);
        self.push_front(i);
    }
    // ANCHOR_END: links

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

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
        cache.put("c", 3);

        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn getting_a_key_refreshes_its_recency() {
        let mut cache = LruCache::new(2);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.get(&"a");
        cache.put("c", 3);

        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn putting_an_existing_key_updates_and_refreshes_it() {
        let mut cache = LruCache::new(2);

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("a", 10);
        cache.put("c", 3);

        assert_eq!(cache.get(&"a"), Some(&10));
        assert_eq!(cache.get(&"b"), None);
        assert_eq!(cache.get(&"c"), Some(&3));
    }

    #[test]
    fn survives_a_longer_sequence_of_operations() {
        let mut cache = LruCache::new(3);
        for i in 0..100 {
            cache.put(i, i * 2);
        }
        // Only the last three keys should remain.
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.get(&99), Some(&198));
        assert_eq!(cache.get(&97), Some(&194));
        assert_eq!(cache.get(&96), None);
    }
    // ANCHOR_END: test
}
