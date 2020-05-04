use std::collections::{linked_list, HashMap};
use std::hash::Hash;
use linked_hash_set::{LinkedHashSet, IntoIter};
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt::Debug;
use std::ops::Index;



pub struct LFUCache<K: Hash + Eq, V> {
    values: HashMap<Rc<K>, ValueCounter<V>>,
    frequency_key_list: RefCell<HashMap<usize, LinkedHashSet<Rc<K>>>>,
    capacity: usize,
    min_frequency: RefCell<usize>,
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
}


impl<K: Hash + Eq, V> LFUCache<K, V> {
    pub fn new(capacity: usize) -> LFUCache<K, V> {
        if capacity == 0 {
            panic!("invalid capacity")
        }
        LFUCache {
            values: HashMap::new(),
            frequency_key_list: RefCell::new(HashMap::new()),
            capacity,
            min_frequency: RefCell::new(0),
        }
    }

    pub fn update_capacity(&mut self, capacity: usize) {
        if capacity < self.capacity {
            panic!("Don't do this");
        }
        self.capacity = capacity;
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let v = self.values.get(&key)?.value();
        self.update_key_frequency(Rc::new(key));
        return Some(v);
    }

    fn update_key_frequency(&self, key: Rc<K>) {
        let count = self.values.get(&key).unwrap().count();
        let mut map = self.frequency_key_list.borrow_mut();
        map.entry(count).or_default().remove(&key);
        if count == *self.min_frequency.borrow() && map.len() == 0 {
            *self.min_frequency.borrow_mut() += 1;
        }
        map.entry(count + 1).or_default().insert(key);
    }


    pub fn set(&mut self, key: K, value: V) {
        let key = Rc::new(key);
        if self.values.contains_key(&key) {
            self.update_key_frequency(Rc::clone(&key));
            let mut valueCounter = self.values.
                get_mut(&Rc::clone(&key))
                .unwrap();
            valueCounter.count += 1;
            return;
        }
        if self.values.len() >= self.capacity {
            let mut temp = self.frequency_key_list.borrow_mut();
            let evict = temp.get_mut(&self.min_frequency.borrow()).unwrap();
            let first = evict.pop_front().unwrap();
            self.values.remove(&first);
        }
        self.values.insert(Rc::clone(&key), ValueCounter { value, count: 1 });
        *self.min_frequency.borrow_mut() = 1;
        self.frequency_key_list.borrow_mut().entry(1).or_default().insert(key);
    }
}


impl<'a, K: Hash + Eq, V> Iterator for &'a LFUCache<K, V> {
    type Item = (&'a Rc<K>, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        for (k, v) in self.values.iter() {
            return Some((k, &v.value));
        }
        return None;
    }
}

impl<'a, K: Hash + Eq, V> Index<K> for LFUCache<K, V> {
    type Output = V;
    fn index(&self, index: K) -> &Self::Output {
        self.get(index).unwrap()
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
        assert_eq!(lfu.get(10).unwrap(), &10);
        assert_eq!(lfu.get(30), None);
    }

    #[test]
    fn test_lru_eviction() {
        let mut lfu = LFUCache::new(2);
        lfu.set(1, 1);
        lfu.set(2, 2);
        lfu.set(3, 3);
        assert_eq!(lfu.get(1), None)
    }

    #[test]
    fn test_key_frequency_update() {
        let mut lfu = LFUCache::new(2);
        lfu.set(1, 1);
        lfu.set(2, 2);
        lfu.set(1, 3);
        dbg!(&lfu.min_frequency);

        lfu.set(10, 10);
        assert_eq!(lfu.get(2), None)
    }


    #[test]
    fn test_lfu_indexing() {
        let mut lfu: LFUCache<i32, i32> = LFUCache::new(2);
        lfu.set(1, 1);
        assert_eq!(lfu[1], 1);
    }
}
