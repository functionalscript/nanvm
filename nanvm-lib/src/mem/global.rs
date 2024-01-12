use core::{alloc::Layout, cell::Cell};
use std::alloc::{alloc, dealloc};

use super::manager::{Dealloc, Manager};

#[derive(Debug, Clone)]
pub struct Global();

pub const GLOBAL: Global = Global();

impl Dealloc for Global {
    type BlockHeader = Cell<isize>;
    #[inline(always)]
    unsafe fn dealloc(ptr: *mut u8, layout: Layout) {
        dealloc(ptr, layout)
    }
}

impl Manager for Global {
    type Dealloc = Global;
    #[inline(always)]
    unsafe fn alloc(self, layout: Layout) -> *mut u8 {
        alloc(layout)
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{fixed::Fixed, manager::Manager};

    use super::GLOBAL;

    #[test]
    #[wasm_bindgen_test]
    fn test_i32() {
        let _x = GLOBAL.fixed_new(Fixed(0));
    }

    struct X<'a>(&'a mut i32);

    impl Drop for X<'_> {
        fn drop(&mut self) {
            *self.0 += 1;
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_x() {
        let mut i = 0;
        assert_eq!(i, 0);
        {
            let _ = GLOBAL.fixed_new(X(&mut i));
        }
        assert_eq!(i, 1);
    }
}
