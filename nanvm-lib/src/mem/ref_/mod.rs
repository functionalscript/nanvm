pub mod update;

use core::{mem::forget, ops::Deref};

use self::update::RefUpdate;

use super::{
    block::{header::BlockHeader, Block},
    manager::Dealloc,
    mut_ref::MutRef,
    object::Object,
};

/// A reference to an object allocated by a memory manager.
#[repr(transparent)]
#[derive(Debug)]
pub struct Ref<T: Object, D: Dealloc> {
    ptr: *const Block<T, D>,
}

impl<T: Object, D: Dealloc> Ref<T, D> {
    #[inline(always)]
    pub unsafe fn new(ptr: *const Block<T, D>) -> Self {
        Self { ptr }
    }
    #[inline(always)]
    unsafe fn ref_update(&self, i: RefUpdate) -> isize {
        (*self.ptr).header.ref_update(i)
    }
    unsafe fn update(&self, i: RefUpdate) -> Option<*mut Block<T, D>> {
        if self.ref_update(i) == 0 {
            Some(self.ptr as *mut _)
        } else {
            None
        }
    }
    pub fn try_to_mut_ref(self) -> Result<MutRef<T, D>, Self> {
        unsafe {
            if let Some(ptr) = self.update(RefUpdate::Read) {
                forget(self);
                Ok(MutRef::new(ptr))
            } else {
                Err(self)
            }
        }
    }
}

impl<T: Object, D: Dealloc> Clone for Ref<T, D> {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            self.ref_update(RefUpdate::AddRef);
            Self { ptr: self.ptr }
        }
    }
}

impl<T: Object, D: Dealloc> Drop for Ref<T, D> {
    fn drop(&mut self) {
        unsafe {
            if let Some(ptr) = self.update(RefUpdate::Release) {
                (*ptr).delete();
            }
        }
    }
}

impl<T: Object, D: Dealloc> Deref for Ref<T, D> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { (*self.ptr).object() }
    }
}

#[cfg(test)]
mod test {
    use core::{alloc::Layout, mem::forget};

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{
        atomic_counter::AtomicCounter,
        block::{header::BlockHeader, Block},
        fixed::Fixed,
        manager::{Dealloc, Manager},
        ref_::update::RefUpdate,
    };

    use super::Ref;

    #[derive(Default)]
    struct BH();

    struct M();

    impl Dealloc for M {
        type BlockHeader = BH;
        unsafe fn dealloc(_: *mut u8, _: core::alloc::Layout) {
            panic!()
        }
    }

    impl Manager for M {
        type Dealloc = Self;
        unsafe fn alloc(self, _: core::alloc::Layout) -> *mut u8 {
            panic!()
        }
    }

    impl BlockHeader for BH {
        unsafe fn ref_update(&self, _: super::update::RefUpdate) -> isize {
            panic!()
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let mut buffer: [(); 0] = [];
        let x = buffer.as_mut_ptr() as *mut Block<Fixed<()>, M>;
        let y = unsafe { Ref::new(x) };
        forget(y);
    }

    struct M1();

    impl Dealloc for M1 {
        type BlockHeader = AtomicCounter;
        unsafe fn dealloc(_: *mut u8, _: Layout) {}
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_1() {
        let mut buffer: [isize; 1] = [0];
        let x = buffer.as_mut_ptr() as *mut Block<Fixed<()>, M1>;
        let p = unsafe { &mut (*x).header };
        assert_eq!(unsafe { p.ref_update(RefUpdate::Read) }, 0);
        {
            let y = unsafe { Ref::new(x) };
            assert_eq!(unsafe { p.ref_update(RefUpdate::Read) }, 0);
            {
                let z = y.clone();
                assert_eq!(unsafe { p.ref_update(RefUpdate::Read) }, 1);
            }
            assert_eq!(unsafe { p.ref_update(RefUpdate::Read) }, 0);
        }
        assert_eq!(unsafe { p.ref_update(RefUpdate::Read) }, -1);
    }
}
