use core::sync::atomic::{AtomicIsize, Ordering};

use crate::mem::{block::header::BlockHeader, ref_::update::RefUpdate};

use super::Global;

pub struct GlobalHeader(AtomicIsize);

impl Default for GlobalHeader {
    fn default() -> Self {
        Self(AtomicIsize::new(1))
    }
}

impl BlockHeader for GlobalHeader {
    type Manager = Global;
    #[inline(always)]
    unsafe fn ref_update(&mut self, val: RefUpdate) -> isize {
        self.0.fetch_add(val as isize, Ordering::Relaxed)
    }
}
