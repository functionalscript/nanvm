pub mod counter_update;

use core::{mem::forget, ops::Deref};

use self::counter_update::RefCounterUpdate;

use super::{
    block::{header::BlockHeader, Block},
    manager::Dealloc,
    mut_ref::MutRef,
    object::Object, variant::Variant,
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
    unsafe fn counter_update(&self, i: RefCounterUpdate) -> Option<*mut Block<T, D>> {
        let ptr = self.ptr;
        if (*ptr).header.ref_counter_update(i) == 0 {
            Some(ptr as *mut _)
        } else {
            None
        }
    }
    pub fn try_to_mut_ref(self) -> Result<MutRef<T, D>, Self> {
        unsafe {
            if let Some(ptr) = self.counter_update(RefCounterUpdate::Read) {
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
            self.counter_update(RefCounterUpdate::AddRef);
            Self { ptr: self.ptr }
        }
    }
}

impl<T: Object, D: Dealloc> Drop for Ref<T, D> {
    fn drop(&mut self) {
        unsafe {
            if let Some(ptr) = self.counter_update(RefCounterUpdate::Release) {
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

#[repr(transparent)]
struct VariantRef<T: Variant> {
    value: T,
}

impl<T: Variant> Clone for VariantRef<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        unsafe { self.value.ref_counter_update(RefCounterUpdate::AddRef) };
        Self { value: self.value }
    }
}

impl<T: Variant> Drop for VariantRef<T> {
    fn drop(&mut self) {
        unsafe {
            if let Some(header) = self.value.ref_counter_update(RefCounterUpdate::Release) {
                self.value.delete(header);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use core::{alloc::Layout, mem::forget, sync::atomic::AtomicIsize};

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{
        block::{header::BlockHeader, Block},
        fixed::Fixed,
        manager::Dealloc,
        ref_::counter_update::RefCounterUpdate,
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

    impl BlockHeader for BH {
        unsafe fn ref_counter_update(&self, _: super::counter_update::RefCounterUpdate) -> isize {
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
        type BlockHeader = AtomicIsize;
        unsafe fn dealloc(_: *mut u8, _: Layout) {}
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_1() {
        let mut buffer: [isize; 1] = [0];
        let x = buffer.as_mut_ptr() as *mut Block<Fixed<()>, M1>;
        let p = unsafe { &mut (*x).header };
        assert_eq!(unsafe { p.ref_counter_update(RefCounterUpdate::Read) }, 0);
        {
            let y = unsafe { Ref::new(x) };
            assert_eq!(unsafe { p.ref_counter_update(RefCounterUpdate::Read) }, 0);
            {
                let z = y.clone();
                assert_eq!(unsafe { p.ref_counter_update(RefCounterUpdate::Read) }, 1);
            }
            assert_eq!(unsafe { p.ref_counter_update(RefCounterUpdate::Read) }, 0);
        }
        assert_eq!(unsafe { p.ref_counter_update(RefCounterUpdate::Read) }, -1);
    }
}
