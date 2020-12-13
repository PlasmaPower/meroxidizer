use crate::{
    bls::{SecretKey, SIG_SIZE},
    cli::Opts,
    rpc::Rpc,
};
use crossbeam_channel::{bounded, RecvTimeoutError, Sender};
use hashbrown::HashMap;
use log::{debug, info, trace, warn};
use meroxidizer::utils::difficulty_to_max_hash;
use parking_lot::RwLock;
use randomx::{Cache, HASH_SIZE};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{self, AtomicUsize},
        Arc,
    },
    thread::JoinHandle,
    time::Duration,
};

pub type Nonce = u32;

pub struct BlockTemplate {
    pub seq: usize,
    pub header: Vec<u8>,
    pub randomx_cache: Arc<Cache>,
    pub max_hash: [u8; 32],
    pub height: usize,
    id: i64,
}

pub struct RpcInfo {
    pub miner_key: SecretKey,
    pub latest_template: RwLock<Arc<BlockTemplate>>,
    pub latest_seq: AtomicUsize,
    /// Channel of (seq, nonce)
    pub publish_channel: Sender<(usize, Nonce, [u8; SIG_SIZE], [u8; HASH_SIZE])>,
    /// Measured in units of `HASH_BATCH_SIZE`
    pub num_hashes_rec: AtomicUsize,
}

const GET_TEMPLATE_INTERVAL: Duration = Duration::from_secs(1);
const RETAIN_SEQS: usize = 5;

pub fn start(opts: Opts) -> (Arc<RpcInfo>, JoinHandle<()>) {
    let mut rpc = Rpc::connect(opts.clone());
    let miner_key = match std::env::var("MEROS_MINER_KEY") {
        Ok(s) => hex::decode(s).expect("Failed to decode MEROS_MINER_KEY env var"),
        Err(std::env::VarError::NotPresent) => {
            match rpc.single_request::<_, String>("personal_getMiner", [(); 0]) {
                Ok(miner) => hex::decode(&miner).expect("Failed to decode miner key from RPC"),
                Err(err) => panic!("Failed to get miner key from RPC: {}", err),
            }
        }
        Err(err) => {
            panic!(
                "Failed to get MEROS_MINER_KEY environment variable: {}",
                err,
            );
        }
    };
    let miner_key = SecretKey::new(&miner_key).expect("Invalid miner key specified");
    let miner_pubkey = hex::encode_upper(miner_key.get_public_key());
    let height = rpc.get_height();
    let target = rpc.get_mining_target(&miner_pubkey);
    info!("loaded miner public key {}", miner_pubkey);

    info!("initializing RandomX..");
    let cache = Cache::new(
        opts.get_randomx_flags(),
        &target.key,
        opts.randomx_init_threads,
    )
    .unwrap();
    info!("initialized RandomX");

    let (publish_send, publish_recv) = bounded(64);
    let mut last_template = Arc::new(BlockTemplate {
        seq: 0,
        header: target.header,
        randomx_cache: Arc::new(cache),
        max_hash: difficulty_to_max_hash(target.difficulty),
        height,
        id: target.id,
    });
    debug!(
        "got first block template with seq {}, id {}, and header {}",
        0,
        last_template.id,
        hex::encode_upper(&last_template.header),
    );
    let rpc_info = Arc::new(RpcInfo {
        miner_key,
        latest_template: RwLock::new(last_template.clone()),
        latest_seq: AtomicUsize::new(0),
        publish_channel: publish_send,
        num_hashes_rec: AtomicUsize::new(0),
    });

    let mut recent_seqs = VecDeque::new();
    recent_seqs.push_back(0);
    let mut last_seq = 0;
    let mut seqs_to_templates = HashMap::new();
    seqs_to_templates.insert(0, last_template.clone());
    let mut last_randomx_key = target.key;
    let rpc_info2 = rpc_info.clone();
    let background = std::thread::spawn(move || loop {
        match publish_recv.recv_timeout(GET_TEMPLATE_INTERVAL) {
            Ok((seq, nonce, signature, hash)) => {
                if let Some(template) = seqs_to_templates.get(&seq) {
                    info!("found block! hash: {}", hex::encode_upper(hash));
                    let mut contents = template.header.clone();
                    contents.extend(&nonce.to_le_bytes());
                    contents.extend(&signature);
                    let params = (template.id, hex::encode_upper(contents));
                    debug!("attempting to publish block with params {:?}", params);
                    let res: Result<bool, _> = rpc.single_request("merit_publishBlock", params);
                    match res {
                        Ok(true) => debug!("successfully published block :)"),
                        Ok(false) => warn!("failed to publish block for unknown reason :("),
                        Err(err) => warn!("failed to publish block :( error: {}", err),
                    }
                    // Empty publish channel as previous blocks aren't useful
                    while let Ok(_) = publish_recv.try_recv() {}
                } else {
                    warn!("found block with expired seq :(");
                    continue;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => return,
        }
        let height = rpc.get_height();
        if height > last_template.height {
            recent_seqs.clear();
            seqs_to_templates.clear();
        }
        let target = rpc.get_mining_target(&miner_pubkey);
        last_seq += 1;
        let mut template = BlockTemplate {
            seq: last_seq,
            header: target.header,
            randomx_cache: last_template.randomx_cache.clone(),
            max_hash: difficulty_to_max_hash(target.difficulty),
            height,
            id: target.id,
        };
        debug!(
            "got new block template with seq {}, id {}, and header {}",
            last_seq,
            template.id,
            hex::encode_upper(&template.header),
        );
        if target.key != last_randomx_key {
            last_randomx_key = target.key;
            if opts.randomx_stop_for_rekey {
                let mut template_lock = rpc_info2.latest_template.write();
                rpc_info2
                    .latest_seq
                    .store(template.seq, atomic::Ordering::Relaxed);
                info!("new RandomX key! waiting for mining threads to pause..");
                drop(last_template);
                drop(template.randomx_cache);
                recent_seqs.clear();
                seqs_to_templates.clear();
                let mut last_counts = (0, 0);
                loop {
                    if let Some(cache) = Arc::get_mut(&mut template_lock)
                        .and_then(|t| Arc::get_mut(&mut t.randomx_cache))
                    {
                        info!("reinitializing RandomX..");
                        cache
                            .set_key(&target.key, opts.randomx_init_threads)
                            .unwrap();
                        info!("reinitialized RandomX");
                        break;
                    } else {
                        if log::log_enabled!(log::Level::Trace) {
                            let new_counts = (
                                Arc::strong_count(&template_lock),
                                Arc::strong_count(&template_lock.randomx_cache),
                            );
                            if last_counts != new_counts {
                                trace!("current refcounts: {} {}", new_counts.0, new_counts.1);
                                last_counts = new_counts;
                            }
                        }
                        std::thread::yield_now();
                    }
                }
                template.randomx_cache = template_lock.randomx_cache.clone();
                last_template = Arc::new(template);
                *template_lock = last_template.clone();
            } else {
                info!("new key! reinitializing RandomX..");
                template.randomx_cache = Arc::new(
                    Cache::new(
                        opts.get_randomx_flags(),
                        &target.key,
                        opts.randomx_init_threads,
                    )
                    .unwrap(),
                );
                info!("reinitialized RandomX");
                last_template = Arc::new(template);
                *rpc_info2.latest_template.write() = last_template.clone();
                rpc_info2
                    .latest_seq
                    .store(last_template.seq, atomic::Ordering::Relaxed);
            }
        } else {
            last_template = Arc::new(template);
            *rpc_info2.latest_template.write() = last_template.clone();
            rpc_info2
                .latest_seq
                .store(last_template.seq, atomic::Ordering::Relaxed);
        }
        seqs_to_templates.insert(last_seq, last_template.clone());
        recent_seqs.push_back(last_seq);
        if recent_seqs.len() > RETAIN_SEQS {
            seqs_to_templates.remove(&recent_seqs.pop_front().unwrap());
        }
    });

    (rpc_info, background)
}
