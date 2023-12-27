use core::{
    cell::Cell,
    sync::atomic::{AtomicIsize, Ordering},
};

use crate::{
    common::ref_mut::RefMut,
    mem::{manager::Dealloc, object::Object, ref_counter_update::RefCounterUpdate},
};

use super::block::Block;

pub trait BlockHeader: Default + Sized {
    // required
    unsafe fn ref_counter_update(&self, i: RefCounterUpdate) -> isize;
    //
    #[inline(always)]
    unsafe fn block<T: Object, D: Dealloc>(&mut self) -> &mut Block<T, D> {
        &mut *(self.to_mut_ptr() as *mut _)
    }
}

impl BlockHeader for AtomicIsize {
    #[inline(always)]
    unsafe fn ref_counter_update(&self, val: RefCounterUpdate) -> isize {
        self.fetch_add(val as isize, Ordering::Relaxed)
    }
}

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
    use core::{
        cell::Cell,
        sync::atomic::{AtomicIsize, Ordering},
    };

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::ref_counter_update::RefCounterUpdate;

    use super::BlockHeader;

    #[test]
    #[wasm_bindgen_test]
    fn test_atomic() {
        let x = AtomicIsize::default();
        assert_eq!(x.load(Ordering::Relaxed), 0);
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

    #[test]
    #[wasm_bindgen_test]
    fn test_cell() {
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
