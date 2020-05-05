
extern crate lfu;

use std::rc::Rc;
use std::collections::HashMap;


fn main() {
    let mut lfu = lfu::LFUCache::new(20);
    lfu.set(10, "niofe".to_string());
    let x = Rc::new("ifoneofe");
    let mut map = HashMap::new();
    map.insert(x, 20);
}