pub mod update;

use core::{mem::forget, ops::Deref};

use self::update::RefUpdate;

use super::{
    block::{header::BlockHeader, Block},
    manager::Manager,
    mut_ref::MutRef,
    object::Object,
};

/// A reference to an object allocated by a memory manager.
#[repr(transparent)]
#[derive(Debug)]
pub struct Ref<T: Object, M: Manager> {
    p: *mut Block<M, T>,
}

impl<T: Object, M: Manager> Ref<T, M> {
    #[inline(always)]
    pub unsafe fn new(p: *mut Block<M, T>) -> Self {
        Self { p }
    }
    #[inline(always)]
    unsafe fn ref_update(&self, i: RefUpdate) -> isize {
        (*self.p).header.ref_update(i)
    }
    pub fn try_to_mut_ref(mut self) -> Result<MutRef<T, M>, Self> {
        unsafe {
            if self.ref_update(RefUpdate::Read) == 0 {
                let result = MutRef::new(self.p);
                forget(self);
                Ok(result)
            } else {
                Err(self)
            }
        }
    }
}

impl<T: Object, M: Manager> Clone for Ref<T, M> {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe {
            self.ref_update(RefUpdate::AddRef);
            Self { p: self.p }
        }
    }
}

impl<T: Object, M: Manager> Drop for Ref<T, M> {
    fn drop(&mut self) {
        unsafe {
            if self.ref_update(RefUpdate::Release) == 0 {
                (*self.p).delete();
            }
        }
    }
}

impl<T: Object, M: Manager> Deref for Ref<T, M> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { (*self.p).object() }
    }
}

#[cfg(test)]
mod test {
    use core::mem::forget;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{
        atomic_counter::AtomicCounter,
        block::{header::BlockHeader, Block},
        fixed::Fixed,
        manager::Manager,
        ref_::update::RefUpdate,
    };

    use super::Ref;

    #[derive(Default)]
    struct BH();

    struct M();

    impl Manager for M {
        type BlockHeader = BH;
        unsafe fn alloc(self, _: core::alloc::Layout) -> *mut u8 {
            panic!()
        }
        unsafe fn dealloc(_: *mut u8, _: core::alloc::Layout) {
            panic!()
        }
    }

    impl BlockHeader for BH {
        unsafe fn ref_update(&mut self, _: super::update::RefUpdate) -> isize {
            panic!()
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let mut buffer: [(); 0] = [];
        let x = buffer.as_mut_ptr() as *mut Block<M, Fixed<()>>;
        let y = unsafe { Ref::new(x) };
        forget(y);
    }

    struct M1();

    impl Manager for M1 {
        type BlockHeader = AtomicCounter;
        unsafe fn alloc(self, _: core::alloc::Layout) -> *mut u8 {
            panic!()
        }
        unsafe fn dealloc(_: *mut u8, _: core::alloc::Layout) {}
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_1() {
        let mut buffer: [isize; 1] = [0];
        let x = buffer.as_mut_ptr() as *mut Block<M1, Fixed<()>>;
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
