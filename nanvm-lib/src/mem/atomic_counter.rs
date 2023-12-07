use core::sync::atomic::{AtomicIsize, Ordering};

use crate::mem::{block::header::BlockHeader, ref_::update::RefUpdate};

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
    unsafe fn ref_update(&mut self, val: RefUpdate) -> isize {
        self.counter.fetch_add(val as isize, Ordering::Relaxed)
    }
}

#[cfg(test)]
mod test {
    use core::sync::atomic::Ordering;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{
        atomic_counter::AtomicCounter, block::header::BlockHeader, ref_::update::RefUpdate,
    };

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let mut x = AtomicCounter::default();
        assert_eq!(x.counter.load(Ordering::Relaxed), 0);
        assert_eq!(unsafe { x.ref_update(RefUpdate::Read) }, 0);
        assert_eq!(unsafe { x.ref_update(RefUpdate::AddRef) }, 0);
        assert_eq!(unsafe { x.ref_update(RefUpdate::Release) }, 1);
        assert_eq!(unsafe { x.ref_update(RefUpdate::Read) }, 0);
        assert_eq!(unsafe { x.ref_update(RefUpdate::Release) }, 0);
        assert_eq!(unsafe { x.ref_update(RefUpdate::Read) }, -1);
    }
}
