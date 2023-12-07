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

#[cfg(test)]
mod test {
    use core::sync::atomic::Ordering;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{block::header::BlockHeader, ref_::update::RefUpdate};

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let mut x = super::GlobalHeader::default();
        assert_eq!(x.counter.load(Ordering::Relaxed), 1);
        assert_eq!(unsafe { x.ref_update(RefUpdate::Read) }, 1);
        assert_eq!(unsafe { x.ref_update(RefUpdate::AddRef) }, 1);
        assert_eq!(unsafe { x.ref_update(RefUpdate::Release) }, 2);
        assert_eq!(unsafe { x.ref_update(RefUpdate::Read) }, 1);
    }
}
