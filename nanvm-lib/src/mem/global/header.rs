use core::sync::atomic::{AtomicIsize, Ordering};

use crate::mem::{block::header::BlockHeader, ref_::update::RefUpdate};

use super::Global;

#[repr(transparent)]
pub struct GlobalHeader {
    counter: AtomicIsize,
}

impl Default for GlobalHeader {
    fn default() -> Self {
        Self {
            counter: AtomicIsize::new(1),
        }
    }
}

impl BlockHeader for GlobalHeader {
    type Manager = Global;
    #[inline(always)]
    unsafe fn ref_update(&mut self, val: RefUpdate) -> isize {
        self.counter.fetch_add(val as isize, Ordering::Relaxed)
    }
}
