use crate::cli::Opts;
use crossbeam_channel::bounded;
use std::thread::JoinHandle;

mod first_hasher;
mod info;
mod rpc_manager;
mod second_hasher;
mod signer;

const HASH_CHAN_BATCH_SIZE: usize = 64;
const HASH_CHAN_CAPACITY: usize = 2;

pub fn start(opts: Opts) -> JoinHandle<()> {
    if opts.bls_threads == 0 || opts.randomx_threads == 0 || opts.randomx_init_threads == 0 {
        eprintln!("You must specify a positive number of each thread type");
        std::process::exit(1);
    }
    if opts.randomx_threads % 2 != 0 {
        eprintln!(
            "You must specify an even number of RandomX threads ({} specified)",
            opts.randomx_threads,
        );
        std::process::exit(1);
    }
    let (rpc_info, handle) = rpc_manager::start(opts.clone());
    let (first_input, first_output) = bounded(HASH_CHAN_CAPACITY);
    for _ in 0..(opts.randomx_threads / 2) {
        first_hasher::start(rpc_info.clone(), first_input.clone());
    }
    let (second_input, second_output) = bounded(HASH_CHAN_CAPACITY);
    for _ in 0..opts.bls_threads {
        signer::start(rpc_info.clone(), first_output.clone(), second_input.clone());
    }
    for _ in 0..(opts.randomx_threads / 2) {
        second_hasher::start(rpc_info.clone(), second_output.clone());
    }
    if opts.output_hash_rate {
        info::start(rpc_info);
    }
    handle
}
