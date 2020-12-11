use super::{rpc_manager::RpcInfo, HASH_CHAN_BATCH_SIZE};
use crate::bls::SIG_SIZE;
use crossbeam_channel::{Receiver, Sender};
use randomx::HASH_SIZE;
use std::sync::Arc;

fn run(
    rpc_info: Arc<RpcInfo>,
    input: Receiver<[(usize, u32, [u8; HASH_SIZE]); HASH_CHAN_BATCH_SIZE]>,
    output: Sender<[(usize, u32, [u8; HASH_SIZE + SIG_SIZE]); HASH_CHAN_BATCH_SIZE]>,
) {
    for items in input {
        let mut out = [(0, 0, [0u8; HASH_SIZE + SIG_SIZE]); HASH_CHAN_BATCH_SIZE];
        for (item_in, item_out) in items.iter().zip(out.iter_mut()) {
            item_out.0 = item_in.0;
            item_out.1 = item_in.1;
            item_out.2[..HASH_SIZE].copy_from_slice(&item_in.2);
            let sig = rpc_info.miner_key.sign(&item_in.2);
            item_out.2[HASH_SIZE..].copy_from_slice(&sig);
        }
        if output.send(out).is_err() {
            return;
        }
    }
}

pub fn start(
    rpc_info: Arc<RpcInfo>,
    input: Receiver<[(usize, u32, [u8; HASH_SIZE]); HASH_CHAN_BATCH_SIZE]>,
    output: Sender<[(usize, u32, [u8; HASH_SIZE + SIG_SIZE]); HASH_CHAN_BATCH_SIZE]>,
) {
    std::thread::spawn(|| run(rpc_info, input, output));
}
