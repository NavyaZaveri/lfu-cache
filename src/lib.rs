//! A Least Frequently Used Cache implementation
//!
//!
//!
//!
//! # Examples
//!
//! ```
//! extern crate lfu;
//! use lfu::LFUCache;
//!
//! # fn main() {
//! let mut lfu = LFUCache::new(2); //initialize an lfu with a maximum capacity of 2 entries
//! lfu.set(2, 2);
//! lfu.set(3, 3);
//! lfu.set(3, 30);
//! lfu.set(4,4); //We're at fully capacity. Purge (2,2) since it's the least-recently-used entry, then insert the current entry

//! assert_eq!(lfu.get(&2), None);
//! assert_eq!(lfu.get(&3), Some(&30));
//!
//! # }
//! ```

use std::collections::HashMap;
use std::hash::Hash;
use linked_hash_set::LinkedHashSet;
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt::Debug;
use std::ops::Index;


#[derive(Debug)]
pub struct LFUCache<K: Hash + Eq, V> {
    values: HashMap<Rc<K>, ValueCounter<V>>,
    frequency_bin: HashMap<usize, LinkedHashSet<Rc<K>>>,
    capacity: usize,
    min_frequency: usize,
}


#[derive(Debug)]
struct ValueCounter<V> {
    value: V,
    count: usize,
}

impl<V> ValueCounter<V> {
    fn value(&self) -> &V {
        return &self.value;
    }
    fn count(&self) -> usize {
        return self.count;
    }
    fn inc(&mut self) {
        self.count += 1;
    }
}


impl<K: Hash + Eq, V> LFUCache<K, V> {
    pub fn new(capacity: usize) -> LFUCache<K, V> {
        if capacity == 0 {
            panic!("invalid capacity")
        }
        LFUCache {
            values: HashMap::new(),
            frequency_bin: HashMap::new(),
            capacity,
            min_frequency: 0,
        }
    }

    fn contains(&self, key: &K) -> bool {
        return self.values.contains_key(key);
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    pub fn remove(&mut self, key: K) -> bool {
        let key = Rc::new(key);
        if let Some(valueCounter) = self.values.get(&Rc::clone(&key)) {
            let count = valueCounter.count();
            self.frequency_bin.entry(count).or_default().remove(&Rc::clone(&key));
            self.values.remove(&key);
        }
        return false;
    }


    pub fn get(&mut self, key: &K) -> Option<&V> {
        if !self.contains(&key) {
            return None;
        }

        let key = self.values.get_key_value(key).map(|(r, _)| Rc::clone(r)).unwrap();
        self.update_frequency_bin(Rc::clone(&key));
        self.values.get(&key).map(|x| x.value())
    }

    fn update_frequency_bin(&mut self, key: Rc<K>) {
        let value_counter = self.values.get_mut(&key).unwrap();
        let bin = self.frequency_bin.get_mut(&value_counter.count).unwrap();
        bin.remove(&key);
        let count = value_counter.count();
        value_counter.inc();
        if count == self.min_frequency && bin.len() == 0 {
            self.min_frequency += 1;
        }
        self.frequency_bin.entry(count + 1).or_default().insert(key);
    }

    fn evict(&mut self) {
        let least_frequently_used_keys = self.frequency_bin.get_mut(&self.min_frequency).unwrap();
        let least_recently_used = least_frequently_used_keys.pop_front().unwrap();
        self.values.remove(&least_recently_used);
    }


    pub fn set(&mut self, key: K, value: V) {
        let key = Rc::new(key);
        if let Some(value_counter) = self.values.get_mut(&key) {
            value_counter.value = value;
            self.update_frequency_bin(Rc::clone(&key));
            return;
        }
        if self.len() >= self.capacity {
            self.evict();
        }
        self.values.insert(Rc::clone(&key), ValueCounter { value, count: 1 });
        self.min_frequency = 1;
        self.frequency_bin.entry(self.min_frequency).or_default().insert(key);
    }
}


impl<'a, K: Hash + Eq, V> Iterator for &'a LFUCache<K, V> {
    type Item = (Rc<K>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        for (k, v) in self.values.iter() {
            return Some((Rc::clone(k), &v.value));
        }
        return None;
    }
}

impl<K: Hash + Eq, V> Index<K> for LFUCache<K, V> {
    type Output = V;
    fn index(&self, index: K) -> &Self::Output {
        return self.values.
            get(&Rc::new(index)).
            map(|x| x.value()).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut lfu = LFUCache::new(20);
        lfu.set(10, 10);
        lfu.set(20, 30);
        assert_eq!(lfu.get(&10).unwrap(), &10);
        assert_eq!(lfu.get(&30), None);
    }

    #[test]
    fn test_lru_eviction() {
        let mut lfu = LFUCache::new(2);
        lfu.set(1, 1);
        lfu.set(2, 2);
        lfu.set(3, 3);
        assert_eq!(lfu.get(&1), None)
    }

    #[test]
    fn test_key_frequency_update() {
        let mut lfu = LFUCache::new(2);
        lfu.set(1, 1);
        lfu.set(2, 2);
        lfu.set(1, 3);

        lfu.set(10, 10);
        assert_eq!(lfu.get(&2), None);
        assert_eq!(lfu[10], 10);
    }


    #[test]
    fn test_lfu_indexing() {
        let mut lfu: LFUCache<i32, i32> = LFUCache::new(2);
        lfu.set(1, 1);
        assert_eq!(lfu[1], 1);
    }

    #[test]
    fn test_lfu_deletion() {
        let mut lfu = LFUCache::new(2);
        lfu.set(1, 1);
        lfu.set(2, 2);
        lfu.remove(1);
        assert_eq!(lfu.get(&1), None);
        lfu.set(3, 3);
        lfu.set(4, 4);
        assert_eq!(lfu.get(&2), None);
        assert_eq!(lfu.get(&3), Some(&3));
    }

    #[test]
    fn test_duplicates() {
        let mut lfu = LFUCache::new(2);
        lfu.set(&1, 1);
        lfu.set(&1, 2);
        lfu.set(&1, 3);
        assert_eq!(lfu[&1], 3);
    }
}
