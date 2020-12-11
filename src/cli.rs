use randomx::Flags;

#[derive(structopt::StructOpt, Clone)]
pub struct Opts {
    /// The RPC address and port.
    #[structopt(short = "r", long = "rpc", default_value = "localhost:5133")]
    pub rpc: String,
    /// The number of threads to use to initialize RandomX.
    /// Only matters on startup and on RandomX key change.
    #[structopt(short = "i", long = "randomx-init-threads")]
    pub randomx_init_threads: u64,
    /// The number of threads to use for RandomX. Must be even.
    #[structopt(short = "t", long = "randomx-threads")]
    pub randomx_threads: usize,
    /// The number of threads to use for BLS signing.
    #[structopt(short = "b", long = "bls-threads")]
    pub bls_threads: usize,
    /// If large pages should be used for RandomX.
    /// Requires special configuration at the OS level.
    #[structopt(short = "l", long = "randomx-large-pages")]
    pub randomx_large_pages: bool,
    /// If the hash rate should be logged every 30 seconds.
    #[structopt(short = "o", long = "output-hash-rate")]
    pub output_hash_rate: bool,
    /// If mining should stop when the RandomX key changes.
    /// Advantage: doesn't double memory usage during RandomX key changes.
    /// Disadvantage: stops mining for a few seconds every other day.
    /// But mining would only progress on the old key anyways, which would be useless.
    /// This is a target for future improvement.
    #[structopt(short = "k", long = "randomx-stop-for-rekey")]
    pub randomx_stop_for_rekey: bool,
}

impl Opts {
    pub fn get_randomx_flags(&self) -> Flags {
        let mut flags = Flags::recommended();
        flags.set_full_mem(true);
        flags.set_large_pages(self.randomx_large_pages);
        flags
    }
}
