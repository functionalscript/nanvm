use core::{mem::forget, ops::Deref};

use super::{
    block::Block, manager::Dealloc, mut_ref::MutRef, object::Object, optional_block::OptionalBlock,
    optional_ref::OptionalRef, ref_counter_update::RefCounterUpdate,
};

/// A reference to an object allocated by a memory manager.
pub type Ref<T, D> = OptionalRef<*const Block<T, D>>;

impl<T: Object, D: Dealloc> Ref<T, D> {
    pub fn try_to_mut_ref(self) -> Result<MutRef<T, D>, Self> {
        unsafe {
            if let Some(ptr) = self.internal().ref_counter_update(RefCounterUpdate::Read) {
                forget(self);
                Ok(MutRef::new(ptr as _))
            } else {
                Err(self)
            }
        }
    }
}

impl<T: Object, D: Dealloc> Deref for Ref<T, D> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { (*self.internal()).object() }
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
        ref_counter_update::RefCounterUpdate,
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
        unsafe fn ref_counter_update(&self, _: RefCounterUpdate) -> isize {
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
