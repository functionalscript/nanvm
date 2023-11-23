mod header;
mod ref_;

use core::{
    alloc::GlobalAlloc,
    ptr::{read, write},
};

use std::alloc::System;

use crate::common::fas::FasLayout;

pub use self::header::Header;
pub use self::ref_::Ref;

#[repr(C)]
pub struct Container<T: Header> {
    counter: usize,
    len: usize,
    pub value: T,
}

pub const DROP: bool = false;
pub const CLONE: bool = true;

impl<T: Header> Container<T> {
    const FAS_LAYOUT: FasLayout<Container<T>, T::Item> = FasLayout::new();
    pub unsafe fn alloc(v: T, len: usize) -> *mut Self {
        let p = System.alloc_zeroed(Self::FAS_LAYOUT.layout(len)) as *mut Self;
        let r = &mut *p;
        r.counter = 0;
        r.len = len;
        write(&mut r.value, v);
        p
    }
    pub unsafe fn add_ref(p: *mut Self) {
        (*p).counter += 1;
    }
    pub unsafe fn release(p: *mut Self) {
        let r = &mut *p;
        let c = r.counter;
        if c != 0 {
            r.counter = c - 1;
            return;
        }
        let len = r.len;
        for i in Self::FAS_LAYOUT.get_mut(r, len) {
            read(i);
        }
        read(&r.value);
        System.dealloc(p as *mut u8, Self::FAS_LAYOUT.layout(len));
    }
    #[inline(always)]
    pub unsafe fn update<const ADD: bool>(p: *mut Self) {
        if ADD {
            Self::add_ref(p)
        } else {
            Self::release(p)
        }
    }
}

#[cfg(test)]
mod test {
    use std::{alloc::Layout, ptr::null_mut};

    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    struct DebugClean {
        p: *mut usize,
        len: usize,
    }

    struct DebugItem(u8);

    impl Drop for DebugClean {
        fn drop(&mut self) {
            unsafe {
                *self.p += 1;
            }
        }
    }

    static mut counter: usize = 0;

    impl Drop for DebugItem {
        fn drop(&mut self) {
            unsafe {
                counter += 1;
            }
        }
    }

    impl Header for DebugClean {
        type Item = DebugItem;
        fn len(&self) -> usize {
            self.len
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn sequential_test() {
        unsafe {
            counter = 0;
            let mut i = 0;
            let p = Container::<DebugClean>::alloc(DebugClean { p: &mut i, len: 0 }, 0);
            assert_eq!(i, 0);
            Container::update::<false>(p);
            assert_eq!(i, 1);
            assert_eq!(counter, 0);
        }
        unsafe {
            counter = 0;
            let mut i = 0;
            let p = Container::<DebugClean>::alloc(DebugClean { p: &mut i, len: 9 }, 9);
            assert_eq!((*p).value.len, 9);
            Container::update::<true>(p);
            Container::update::<false>(p);
            assert_eq!(i, 0);
            Container::update::<false>(p);
            assert_eq!(i, 1);
            assert_eq!(counter, 9);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_layout() {
        let cl = Container::<DebugClean>::FAS_LAYOUT;
        let x = cl.layout(9);
        let r = Layout::new::<Container<DebugClean>>()
            .extend(Layout::array::<u8>(9).unwrap())
            .unwrap();
        assert_eq!(r.0, x);
    }
}
