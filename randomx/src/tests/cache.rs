use crate::{Cache, Flags};

#[test]
fn init_cache_only() {
    let mut flags = Flags::recommended();
    flags.set_full_mem(false);
    Cache::new(flags, &[1; 32], 0).unwrap();
}

#[test]
#[ignore = "slow and multi core test should cover it"]
fn init_single_core() {
    let mut flags = Flags::recommended();
    flags.set_full_mem(true);
    Cache::new(flags, &[1; 32], 1).unwrap();
}

#[test]
fn init_multi_core() {
    let mut flags = Flags::recommended();
    flags.set_full_mem(true);
    Cache::new(flags, &[1; 32], 16).unwrap();
}
