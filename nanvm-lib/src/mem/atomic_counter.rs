use core::sync::atomic::{AtomicIsize, Ordering};

use crate::mem::{block::header::BlockHeader, ref_::counter_update::RefCounterUpdate};

#[repr(transparent)]
pub struct AtomicCounter {
    counter: AtomicIsize,
}

impl Default for AtomicCounter {
    #[inline(always)]
    fn default() -> Self {
        Self {
            counter: AtomicIsize::new(0),
        }
    }
}

impl BlockHeader for AtomicCounter {
    #[inline(always)]
    unsafe fn ref_counter_update(&self, val: RefCounterUpdate) -> isize {
        self.counter.fetch_add(val as isize, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod test {
    use core::sync::atomic::Ordering;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{
        atomic_counter::AtomicCounter, block::header::BlockHeader,
        ref_::counter_update::RefCounterUpdate,
    };

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let mut x = AtomicCounter::default();
        assert_eq!(x.counter.load(Ordering::Relaxed), 0);
        assert_eq!(unsafe { x.ref_counter_update(RefCounterUpdate::Read) }, 0);
        assert_eq!(unsafe { x.ref_counter_update(RefCounterUpdate::AddRef) }, 0);
        assert_eq!(
            unsafe { x.ref_counter_update(RefCounterUpdate::Release) },
            1
        );
        assert_eq!(unsafe { x.ref_counter_update(RefCounterUpdate::Read) }, 0);
        assert_eq!(
            unsafe { x.ref_counter_update(RefCounterUpdate::Release) },
            0
        );
        assert_eq!(unsafe { x.ref_counter_update(RefCounterUpdate::Read) }, -1);
    }
}
