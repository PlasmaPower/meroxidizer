use super::{rpc_manager::RpcInfo, PartialHashBatch};
use crate::bls::SIG_SIZE;
use crossbeam_channel::Receiver;
use log::trace;
use randomx::{HashChain, Vm, HASH_SIZE};
use std::{
    cmp::Ordering,
    sync::{atomic, Arc},
};

fn less_than_rev(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut i = 31;
    while i > 0 {
        match a[i].cmp(&b[i]) {
            Ordering::Less => return true,
            Ordering::Greater => return false,
            Ordering::Equal => {}
        }
        i -= 1;
    }
    false
}

fn run(
    rpc_info: Arc<RpcInfo>,
    inputs_chan: Receiver<PartialHashBatch<[u8; HASH_SIZE + SIG_SIZE]>>,
) {
    let mut template = rpc_info.latest_template.read().clone();
    let mut vm = Vm::new(template.randomx_cache.clone()).expect("Failed to create RandomX VM");
    loop {
        let inputs = match inputs_chan.recv() {
            Ok(x) => x,
            Err(_) => return,
        };
        if inputs.height < template.height {
            continue;
        }
        let mut prev_input = inputs.items[0];
        let mut hash_chain = HashChain::new(&mut vm, &prev_input.1);
        trace!("second_hasher loaded template with seq {}", template.seq);
        for input in inputs.items[1..].iter() {
            let out = hash_chain.next(&input.1);
            if less_than_rev(&out, &template.max_hash) {
                let mut sig = [0u8; SIG_SIZE];
                sig.copy_from_slice(&prev_input.1[HASH_SIZE..]);
                let data = (inputs.seq, prev_input.0, sig, out);
                if let Err(_) = rpc_info.publish_channel.send(data) {
                    return;
                }
            }
            prev_input = *input;
        }
        let out = hash_chain.last();
        if less_than_rev(&out, &template.max_hash) {
            let mut sig = [0u8; SIG_SIZE];
            sig.copy_from_slice(&prev_input.1[HASH_SIZE..]);
            let data = (inputs.seq, prev_input.0, sig, out);
            if let Err(_) = rpc_info.publish_channel.send(data) {
                return;
            }
        }
        rpc_info
            .num_hashes_rec
            .fetch_add(1, atomic::Ordering::Relaxed);
        if rpc_info.latest_seq.load(atomic::Ordering::Relaxed) > template.seq {
            std::mem::drop(template);
            let vm_no_cache = vm.drop_cache();
            template = rpc_info.latest_template.read().clone();
            vm = vm_no_cache
                .set_cache(template.randomx_cache.clone())
                .expect("Failed to set RandomX cache");
        }
    }
}

pub fn start(
    rpc_info: Arc<RpcInfo>,
    inputs_chan: Receiver<PartialHashBatch<[u8; HASH_SIZE + SIG_SIZE]>>,
) {
    std::thread::spawn(|| run(rpc_info, inputs_chan));
}
