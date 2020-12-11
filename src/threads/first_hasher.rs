use super::{
    rpc_manager::{Nonce, RpcInfo},
    HASH_CHAN_BATCH_SIZE,
};
use crossbeam_channel::Sender;
use log::trace;
use rand::{thread_rng, Rng};
use randomx::{HashChain, Vm, HASH_SIZE};
use std::sync::{atomic, Arc};

fn run(
    rpc_info: Arc<RpcInfo>,
    output: Sender<[(usize, u32, [u8; HASH_SIZE]); HASH_CHAN_BATCH_SIZE]>,
) {
    let mut template = rpc_info.latest_template.read().clone();
    let mut vm = Vm::new(template.randomx_cache.clone()).expect("Failed to create RandomX VM");
    loop {
        trace!("first_hasher loaded template with seq {}", template.seq);
        let mut outputs_buf = [(0, 0, [0; HASH_SIZE]); HASH_CHAN_BATCH_SIZE];
        let mut input = template.header.clone();
        let mut nonce: Nonce = thread_rng().gen();
        input.extend(&nonce.to_le_bytes());
        let mut hash_chain = HashChain::new(&mut vm, &input);
        for out in &mut outputs_buf[..(HASH_CHAN_BATCH_SIZE - 1)] {
            let prev_nonce = nonce;
            nonce = nonce.wrapping_add(1);
            input[template.header.len()..].copy_from_slice(&nonce.to_le_bytes());
            let prev_hash = hash_chain.next(&input);
            *out = (template.seq, prev_nonce, prev_hash);
        }
        outputs_buf[outputs_buf.len() - 1] = (template.seq, nonce, hash_chain.last());
        if let Err(_) = output.send(outputs_buf) {
            return;
        }
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
    output: Sender<[(usize, u32, [u8; HASH_SIZE]); HASH_CHAN_BATCH_SIZE]>,
) {
    std::thread::spawn(|| run(rpc_info, output));
}
