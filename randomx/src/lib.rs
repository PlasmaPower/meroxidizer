mod cache;
mod error;
mod flags;
mod hash_chain;
mod vm;

#[cfg(test)]
mod tests;

pub use cache::Cache;
pub use error::Error;
pub use flags::Flags;
pub use hash_chain::HashChain;
pub use randomx_sys::RANDOMX_HASH_SIZE as HASH_SIZE;
pub use vm::{Vm, VmWithoutCache};

type Result<T, E = Error> = std::result::Result<T, E>;
