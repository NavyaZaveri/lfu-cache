extern crate lfu;


fn main() {
    let mut lfu = lfu::LFUCache::new(20);
    lfu.set(10, 20);
    let  x= lfu.get(10);
    dbg!(x);
}