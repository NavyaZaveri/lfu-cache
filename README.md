# lfu-cache
A rust implementation of a [Least Frequently Used (LFU)](https://en.wikipedia.org/wiki/Least_frequently_used) cache.


Usage:
```
 extern crate lfu;
 use lfu::LFUCache;

 fn main() {
     let mut lfu = LFUCache::new(2).unwrap(); //initialize an lfu with a maximum capacity of 2 entries
     lfu.set(2, 2);
     lfu.set(3, 3);
     lfu.set(3, 30);
    
    
    //We're at fully capacity. First purge (2,2) since it's the least-frequently-used entry, then insert the current entry
     lfu.set(4,4); 
    
     assert_eq!(lfu.get(&2), None);
     assert_eq!(lfu.get(&3), Some(&30));
}
```
