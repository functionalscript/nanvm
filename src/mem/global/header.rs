use core::{
    alloc::Layout,
    sync::atomic::{AtomicIsize, Ordering},
};

use std::alloc::dealloc;

use crate::mem::{block::header::BlockHeader, ref_::update::RefUpdate};

pub struct GlobalHeader(AtomicIsize);

impl Default for GlobalHeader {
    fn default() -> Self {
        Self(AtomicIsize::new(1))
    }
}

impl BlockHeader for GlobalHeader {
    #[inline(always)]
    unsafe fn ref_update(&mut self, i: RefUpdate) -> isize {
        self.0.fetch_add(i as isize, Ordering::Relaxed)
    }
    unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
        dealloc(ptr, layout);
    }
}
