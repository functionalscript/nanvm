pub mod counter_update;

use core::{mem::forget, ops::Deref};

use self::counter_update::RefCounterUpdate;

use super::{
    block::{header::BlockHeader, Block},
    manager::Dealloc,
    mut_ref::MutRef,
    object::Object,
    variant::Variant,
};

/// A reference to an object allocated by a memory manager.
pub type Ref<T: Object, D: Dealloc> = VariantRef<*const Block<T, D>>;

impl<T: Object, D: Dealloc> Ref<T, D> {
    #[inline(always)]
    pub unsafe fn new(value: *const Block<T, D>) -> Self {
        Self { value }
    }
    pub fn try_to_mut_ref(self) -> Result<MutRef<T, D>, Self> {
        unsafe {
            if let Some(ptr) = self.value.ref_counter_update(RefCounterUpdate::Read) {
                forget(self);
                Ok(MutRef::new(ptr as _))
            } else {
                Err(self)
            }
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct VariantRef<T: Variant> {
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

impl<T: Object, D: Dealloc> Deref for Ref<T, D> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { (*self.value).object() }
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
