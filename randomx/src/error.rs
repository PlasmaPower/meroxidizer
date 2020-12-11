use crate::Flags;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[non_exhaustive]
pub enum Error {
    #[error("failed to allocate cache")]
    AllocCacheFailed,
    #[error("failed to allocate dataset")]
    AllocDatasetFailed,
    #[error("failed to create VM")]
    CreateVmFailed,
    #[error("background thread panicked")]
    ThreadPanic,
    #[error("attempted to initialize full memory dataset with zero threads")]
    ZeroInitThreads,
    #[error("attempted to mix structs with flags {0:?} and {1:?}")]
    FlagsMismatch(Flags, Flags),
}
