use randomx_sys::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Flags(randomx_flags);

impl Flags {
    pub fn recommended() -> Self {
        Flags(unsafe { randomx_get_flags() })
    }

    pub fn get_full_mem(&self) -> bool {
        self.0 & RANDOMX_FLAG_FULL_MEM == RANDOMX_FLAG_FULL_MEM
    }

    pub fn set_full_mem(&mut self, value: bool) {
        if value {
            self.0 |= RANDOMX_FLAG_FULL_MEM;
        } else {
            self.0 &= !RANDOMX_FLAG_FULL_MEM;
        }
    }

    pub fn get_large_pages(&self) -> bool {
        self.0 & RANDOMX_FLAG_LARGE_PAGES == RANDOMX_FLAG_LARGE_PAGES
    }

    pub fn set_large_pages(&mut self, value: bool) {
        if value {
            self.0 |= RANDOMX_FLAG_LARGE_PAGES;
        } else {
            self.0 &= !RANDOMX_FLAG_LARGE_PAGES;
        }
    }
}

impl Into<randomx_flags> for Flags {
    fn into(self) -> randomx_flags {
        self.0
    }
}
