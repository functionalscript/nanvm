use core::cell::Cell;

use crate::mem::ref_::counter_update::RefCounterUpdate;

use super::header::BlockHeader;

impl BlockHeader for Cell<isize> {
    #[inline(always)]
    unsafe fn ref_counter_update(&self, val: RefCounterUpdate) -> isize {
        let result = self.get();
        self.set(result + val as isize);
        result
    }
}

#[cfg(test)]
mod test {
    use core::cell::Cell;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{block::header::BlockHeader, ref_::counter_update::RefCounterUpdate};

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let x = Cell::default();
        assert_eq!(x.get(), 0);
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
