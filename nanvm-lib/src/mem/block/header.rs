use crate::{
    common::ref_mut::RefMut,
    mem::{manager::Dealloc, object::Object, ref_::update::RefUpdate},
};

use super::Block;

pub trait BlockHeader: Default + Sized {
    // required
    unsafe fn ref_update(&mut self, i: RefUpdate) -> isize;
    //
    #[inline(always)]
    unsafe fn block<D: Dealloc, T: Object>(&mut self) -> &mut Block<D, T> {
        &mut *(self.as_mut_ptr() as *mut _)
    }
}

#[cfg(test)]
mod test {
    use core::{alloc::Layout, marker::PhantomData};

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        common::ref_mut::RefMut,
        mem::{
            block::Block,
            fixed::Fixed,
            manager::{Dealloc, Manager},
            ref_::update::RefUpdate,
        },
    };

    use super::BlockHeader;

    struct XBH(isize);

    impl Default for XBH {
        fn default() -> Self {
            Self(0)
        }
    }

    struct D();

    impl Dealloc for D {
        type BlockHeader = XBH;
        unsafe fn dealloc(_: *mut u8, _: Layout) {}
    }

    impl Manager for D {
        type Dealloc = Self;
        unsafe fn alloc(self, layout: Layout) -> *mut u8 {
            todo!()
        }
    }

    impl BlockHeader for XBH {
        unsafe fn ref_update(&mut self, i: RefUpdate) -> isize {
            let result = self.0;
            self.0 += i as isize;
            result
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let mut x = Block::<D, Fixed<()>> {
            header: XBH::default(),
            _0: PhantomData,
        };
        let p = unsafe { x.header.block::<D, Fixed<()>>() };
        unsafe {
            assert_eq!(p.as_mut_ptr(), (&mut x).as_mut_ptr());
        }
        unsafe {
            assert_eq!(x.header.ref_update(RefUpdate::Read), 0);
            assert_eq!(x.header.ref_update(RefUpdate::AddRef), 0);
            assert_eq!(x.header.ref_update(RefUpdate::Read), 1);
        }
    }
}
