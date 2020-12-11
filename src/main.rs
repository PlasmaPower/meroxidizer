use structopt::StructOpt;

mod bls;
mod cli;
mod rpc;
mod threads;

fn main() {
    env_logger::init();
    let opts = cli::Opts::from_args();
    let _ = threads::start(opts).join();
}
