#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate link_cplusplus;

mod inner {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// Overwrite the types of these to be `usize`s.
pub const RANDOMX_HASH_SIZE: usize = inner::RANDOMX_HASH_SIZE as _;
pub const RANDOMX_DATASET_ITEM_SIZE: usize = inner::RANDOMX_DATASET_ITEM_SIZE as _;

pub use inner::*;
