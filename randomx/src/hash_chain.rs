use crate::Vm;
use randomx_sys::*;

pub struct HashChain<'a> {
    vm: &'a mut Vm,
}

impl<'a> HashChain<'a> {
    /// Queues up this input and creates a hash chain.
    pub fn new(vm: &'a mut Vm, input: &[u8]) -> HashChain<'a> {
        unsafe {
            randomx_calculate_hash_first(vm.get_ptr().as_ptr(), input.as_ptr() as _, input.len());
        }
        HashChain { vm }
    }

    /// Returns the output of the _previous_ input, while queueing up this next input.
    pub fn next(&mut self, input: &[u8]) -> [u8; RANDOMX_HASH_SIZE] {
        let mut output = [0u8; RANDOMX_HASH_SIZE];
        unsafe {
            randomx_calculate_hash_next(
                self.vm.get_ptr().as_ptr(),
                input.as_ptr() as _,
                input.len(),
                output.as_mut_ptr() as _,
            );
        }
        output
    }

    /// Ends the chain, returning the output of the last remaining queued input.
    pub fn last(self) -> [u8; RANDOMX_HASH_SIZE] {
        let mut output = [0u8; RANDOMX_HASH_SIZE];
        unsafe {
            randomx_calculate_hash_last(self.vm.get_ptr().as_ptr(), output.as_mut_ptr() as _);
        }
        output
    }

    pub fn get_vm(&self) -> &Vm {
        &self.vm
    }
}
