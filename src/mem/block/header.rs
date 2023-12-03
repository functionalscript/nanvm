use core::alloc::Layout;

use crate::{
    common::ref_mut::RefMut,
    mem::{object::Object, ref_::update::RefUpdate},
};

use super::Block;

pub trait BlockHeader: Default + Sized {
    // required
    unsafe fn ref_update(&mut self, i: RefUpdate) -> isize;
    unsafe fn dealloc(ptr: *mut u8, layout: Layout);
    //
    #[inline(always)]
    unsafe fn block<T: Object>(&mut self) -> &mut Block<Self, T> {
        &mut *(self.as_mut_ptr() as *mut _)
    }
}

#[cfg(test)]
mod test {
    use core::{alloc::Layout, marker::PhantomData};

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        common::ref_mut::RefMut,
        mem::{block::Block, fixed::Fixed, ref_::update::RefUpdate},
    };

    use super::BlockHeader;

    struct XBH(isize);

    impl Default for XBH {
        fn default() -> Self {
            Self(0)
        }
    }

    impl BlockHeader for XBH {
        unsafe fn ref_update(&mut self, i: RefUpdate) -> isize {
            self.0 += i as isize;
            self.0
        }
        unsafe fn dealloc(p: *mut u8, layout: Layout) {}
    }

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let mut x = Block::<XBH, Fixed<()>> {
            header: XBH::default(),
            _0: PhantomData,
        };
        let p = unsafe { x.header.block::<Fixed<()>>() };
        unsafe {
            assert_eq!(p.as_mut_ptr(), (&mut x).as_mut_ptr());
        }
    }
}
