use crate::{Error, Flags, Result};
use randomx_sys::*;
use std::ptr;

/// Combines the RandomX cache and optionally dataset
pub struct Cache {
    cache_ptr: ptr::NonNull<randomx_cache>,
    dataset_ptr: Option<ptr::NonNull<randomx_dataset>>,
    flags: Flags,
}

impl Cache {
    pub fn new(flags: Flags, key: &[u8], init_threads: u64) -> Result<Self> {
        let cache_ptr = unsafe { randomx_alloc_cache(flags.into()) };
        let cache_ptr = ptr::NonNull::new(cache_ptr).ok_or(Error::AllocCacheFailed)?;
        let mut this = Cache {
            cache_ptr,
            dataset_ptr: None,
            flags,
        };
        if flags.get_full_mem() {
            this.dataset_ptr = unsafe { ptr::NonNull::new(randomx_alloc_dataset(flags.into())) };
            if this.dataset_ptr.is_none() {
                return Err(Error::AllocDatasetFailed);
            }
        }
        this.set_key(key, init_threads)?;
        Ok(this)
    }

    pub fn get_flags(&self) -> Flags {
        self.flags
    }

    pub fn get_cache_ptr(&self) -> ptr::NonNull<randomx_cache> {
        self.cache_ptr
    }

    pub fn get_dataset_ptr(&self) -> Option<ptr::NonNull<randomx_dataset>> {
        self.dataset_ptr
    }

    pub fn set_key(&mut self, key: &[u8], threads: u64) -> Result<()> {
        if self.dataset_ptr.is_some() && threads == 0 {
            return Err(Error::ZeroInitThreads);
        }
        unsafe {
            randomx_init_cache(self.cache_ptr.as_ptr(), key.as_ptr() as *const _, key.len());
        }
        if let Some(dataset_ptr) = self.dataset_ptr {
            let count = unsafe { randomx_dataset_item_count() };
            let init_threads = std::cmp::min(threads, count);
            let per_thread = count / init_threads;
            let remainder = count % init_threads;
            let mut threads = Vec::with_capacity(init_threads as usize);
            for i in 0..init_threads {
                let start = i * per_thread + std::cmp::min(i, remainder);
                let mut num = per_thread;
                if i < remainder {
                    num += 1;
                }
                struct UnsafeSend(ptr::NonNull<randomx_dataset>, ptr::NonNull<randomx_cache>);
                unsafe impl Send for UnsafeSend {}
                let ptrs = UnsafeSend(dataset_ptr, self.cache_ptr);
                threads.push(std::thread::spawn(move || unsafe {
                    randomx_init_dataset(ptrs.0.as_ptr(), ptrs.1.as_ptr(), start, num);
                }));
            }
            // Delay errors to make sure that threads finish first, delaying Drop impl.
            let mut thread_res = Ok(());
            for thread in threads {
                let join_res = thread.join();
                thread_res = thread_res.and(join_res);
            }
            thread_res.map_err(|_| Error::ThreadPanic)?;
        }
        Ok(())
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        unsafe {
            if let Some(dataset_ptr) = self.dataset_ptr {
                randomx_release_dataset(dataset_ptr.as_ptr());
            }
            randomx_release_cache(self.cache_ptr.as_ptr());
        }
    }
}

unsafe impl Send for Cache {}
unsafe impl Sync for Cache {}
