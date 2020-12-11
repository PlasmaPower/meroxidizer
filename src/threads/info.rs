use super::{rpc_manager::RpcInfo, HASH_CHAN_BATCH_SIZE};
use log::info;
use std::{
    sync::{atomic, Arc},
    time::Duration,
};

const OUTPUT_INFO_INTERVAL: Duration = Duration::from_secs(30);

fn run(rpc_info: Arc<RpcInfo>) {
    loop {
        std::thread::sleep(OUTPUT_INFO_INTERVAL);
        let new_hashes = rpc_info.num_hashes_rec.swap(0, atomic::Ordering::Relaxed);
        info!(
            "current hashrate: {} H/s",
            (new_hashes * HASH_CHAN_BATCH_SIZE) / (OUTPUT_INFO_INTERVAL.as_secs() as usize),
        );
    }
}

pub fn start(rpc_info: Arc<RpcInfo>) {
    std::thread::spawn(|| run(rpc_info));
}
