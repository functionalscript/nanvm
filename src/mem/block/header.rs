use crate::{
    common::ref_mut::RefMut,
    mem::{object::Object, ref_::update::RefUpdate},
};

use super::Block;

pub trait BlockHeader: Sized {
    // required
    unsafe fn ref_update(&mut self, i: RefUpdate) -> isize;
    unsafe fn delete<T: Object>(block: &mut Block<Self, T>);
    //
    #[inline(always)]
    unsafe fn block<T: Object>(&mut self) -> &mut Block<Self, T> {
        &mut *(self.as_mut_ptr() as *mut _)
    }
}

#[cfg(test)]
mod test {
    use core::marker::PhantomData;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::{
        common::ref_mut::RefMut,
        mem::{block::Block, fixed::Fixed, object::Object, ref_::update::RefUpdate},
    };

    use super::BlockHeader;

    struct XBH(isize);

    impl BlockHeader for XBH {
        unsafe fn ref_update(&mut self, i: RefUpdate) -> isize {
            self.0 += i as isize;
            self.0
        }
        unsafe fn delete<T: Object>(block: &mut Block<Self, T>) {}
    }

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        let mut x = Block::<XBH, Fixed<()>> {
            header: XBH(0),
            _0: PhantomData,
        };
        let p = unsafe { x.header.block::<Fixed<()>>() };
        unsafe {
            assert_eq!(p.as_mut_ptr(), (&mut x).as_mut_ptr());
        }
    }
}
