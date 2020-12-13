use super::{rpc_manager::RpcInfo, HASH_CHAN_BATCH_SIZE, PartialHashBatch};
use crate::bls::SIG_SIZE;
use crossbeam_channel::{Receiver, Sender};
use randomx::HASH_SIZE;
use std::sync::Arc;

fn run(
    rpc_info: Arc<RpcInfo>,
    inputs: Receiver<PartialHashBatch<[u8; HASH_SIZE]>>,
    outputs: Sender<PartialHashBatch<[u8; HASH_SIZE + SIG_SIZE]>>,
) {
    for input in inputs {
        let mut out = PartialHashBatch {
            seq: input.seq,
            height: input.height,
            items: [(0, [0; HASH_SIZE + SIG_SIZE]); HASH_CHAN_BATCH_SIZE],
        };
        for (item_in, item_out) in input.items.iter().zip(out.items.iter_mut()) {
            item_out.0 = item_in.0;
            item_out.1[..HASH_SIZE].copy_from_slice(&item_in.1);
            let sig = rpc_info.miner_key.sign(&item_in.1);
            item_out.1[HASH_SIZE..].copy_from_slice(&sig);
        }
        if outputs.send(out).is_err() {
            return;
        }
    }
}

pub fn start(
    rpc_info: Arc<RpcInfo>,
    inputs: Receiver<PartialHashBatch<[u8; HASH_SIZE]>>,
    outputs: Sender<PartialHashBatch<[u8; HASH_SIZE + SIG_SIZE]>>,
) {
    std::thread::spawn(|| run(rpc_info, inputs, outputs));
}
