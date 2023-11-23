mod header;
mod ref_;

use std::{
    alloc::{GlobalAlloc, Layout, System},
    ptr::read,
};

use crate::common::fas::FasLayout;

pub use self::header::Header;
pub use self::ref_::Ref;

#[repr(C)]
pub struct Container<T: Header> {
    counter: usize,
    pub value: T,
    len: usize,
}

pub const DROP: bool = false;
pub const CLONE: bool = true;

const fn compatible(t: usize, i: Layout) {
    assert!(t >= i.align());
    assert!(t % i.align() == 0);
}

impl<T: Header> Container<T> {
    const FAS_LAYOUT: FasLayout<Container<T>, T::Item> = FasLayout::new();
    pub unsafe fn alloc(len: usize) -> *mut Self {
        let p = System.alloc_zeroed(Self::FAS_LAYOUT.layout(len)) as *mut Self;
        (*p).len = len;
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
        read(&r.value);
        for i in Self::FAS_LAYOUT.get_mut(r, r.len) {
            read(i);
        }
        System.dealloc(p as *mut u8, Self::FAS_LAYOUT.layout(r.len));
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
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    struct DebugClean(*mut usize);

    struct DebugItem(u8);

    impl Drop for DebugClean {
        fn drop(&mut self) {
            unsafe {
                *self.0 += 1;
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
    }

    #[test]
    #[wasm_bindgen_test]
    fn sequential_test() {
        unsafe {
            counter = 0;
            let p = Container::<DebugClean>::alloc(0);
            let mut i = 0;
            (*p).value.0 = &mut i;
            assert_eq!(i, 0);
            Container::update::<false>(p);
            assert_eq!(i, 1);
            assert_eq!(counter, 0);
        }
        unsafe {
            counter = 0;
            let p = Container::<DebugClean>::alloc(9);
            assert_eq!((*p).len, 9);
            let mut i = 0;
            (*p).value.0 = &mut i;
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
