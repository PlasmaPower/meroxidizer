use crate::{cache::Cache, Error, Flags, Result};
use randomx_sys::*;
use std::{ptr, sync::Arc};

pub struct Vm {
    cache: Arc<Cache>,
    ptr: ptr::NonNull<randomx_vm>,
}

fn set_cache(ptr: ptr::NonNull<randomx_vm>, cache: &Cache) {
    if let Some(dataset) = cache.get_dataset_ptr() {
        unsafe {
            randomx_vm_set_dataset(ptr.as_ptr(), dataset.as_ptr());
        }
    } else {
        unsafe {
            randomx_vm_set_cache(ptr.as_ptr(), cache.get_cache_ptr().as_ptr());
        }
    }
}

impl Vm {
    pub fn new(cache: Arc<Cache>) -> Result<Self> {
        let flags = cache.get_flags();
        let (cache_ptr, dataset_ptr) = match cache.get_dataset_ptr() {
            Some(dataset) => (ptr::null_mut(), dataset.as_ptr()),
            None => (cache.get_cache_ptr().as_ptr(), ptr::null_mut()),
        };
        let ptr = unsafe { randomx_create_vm(flags.into(), cache_ptr, dataset_ptr) };
        let ptr = ptr::NonNull::new(ptr).ok_or(Error::CreateVmFailed)?;
        Ok(Vm { cache, ptr })
    }

    pub fn replace_cache(&mut self, new_cache: Arc<Cache>) -> Result<()> {
        let our_flags = self.cache.get_flags();
        let new_flags = new_cache.get_flags();
        if our_flags != new_flags {
            return Err(Error::FlagsMismatch(our_flags, new_flags));
        }
        set_cache(self.ptr, &new_cache);
        self.cache = new_cache;
        Ok(())
    }

    pub fn hash(&mut self, input: &[u8]) -> [u8; RANDOMX_HASH_SIZE] {
        let mut output = [0u8; RANDOMX_HASH_SIZE];
        unsafe {
            randomx_calculate_hash(
                self.ptr.as_ptr(),
                input.as_ptr() as _,
                input.len(),
                output.as_mut_ptr() as _,
            );
        }
        output
    }

    pub fn get_flags(&self) -> Flags {
        self.cache.get_flags()
    }

    pub fn get_cache(&self) -> &Arc<Cache> {
        &self.cache
    }

    pub fn get_ptr(&self) -> ptr::NonNull<randomx_vm> {
        self.ptr
    }

    pub fn drop_cache(self) -> VmWithoutCache {
        VmWithoutCache {
            ptr: self.ptr,
            flags: self.cache.get_flags(),
        }
    }
}

pub struct VmWithoutCache {
    ptr: ptr::NonNull<randomx_vm>,
    flags: Flags,
}

impl VmWithoutCache {
    pub fn set_cache(self, cache: Arc<Cache>) -> Result<Vm> {
        let new_flags = cache.get_flags();
        if self.flags != new_flags {
            return Err(Error::FlagsMismatch(self.flags, new_flags));
        }
        set_cache(self.ptr, &cache);
        Ok(Vm {
            ptr: self.ptr,
            cache,
        })
    }
}
