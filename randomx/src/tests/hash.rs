use crate::{Cache, Flags, HashChain, Vm};
use std::sync::Arc;

#[test]
fn compute_hash() {
    let mut flags = Flags::recommended();
    flags.set_full_mem(false);
    let cache = Arc::new(Cache::new(flags, &[0; 32], 16).unwrap());
    let mut vm = Vm::new(cache).unwrap();
    let hash = vm.hash(b"hello world");
    assert_eq!(
        &hex::encode(hash),
        "f7956d0189fd2f6ca8f6a568447240b19cc381c37a203385dc3f2a8fbd567158",
    );
}

#[test]
fn compute_hash_chain() {
    let mut flags = Flags::recommended();
    flags.set_full_mem(false);
    let cache = Arc::new(Cache::new(flags, &[0; 32], 16).unwrap());
    let mut vm = Vm::new(cache).unwrap();
    let mut chain = HashChain::new(&mut vm, b"hello world");
    let hash0 = chain.next(b"foobar");
    assert_eq!(
        &hex::encode(hash0),
        "f7956d0189fd2f6ca8f6a568447240b19cc381c37a203385dc3f2a8fbd567158",
    );
    let hash1 = chain.last();
    assert_eq!(
        &hex::encode(hash1),
        "d3337885b272abc0b44d8d23056e2ba64e095bc1bc3b195bd09d49089e14f1d2",
    );
}
