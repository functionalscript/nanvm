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
    pub unsafe fn alloc(value: T, items: impl ExactSizeIterator<Item = T::Item>) -> *mut Self {
        let mut len = items.len();
        let p = System.alloc(Self::FAS_LAYOUT.layout(len)) as *mut Self;
        let header = &mut *p;
        write(
            header,
            Container {
                counter: 0,
                len,
                value,
            },
        );
        for (dst, src) in header.get_items_mut().iter_mut().zip(items) {
            write(dst, src);
            len -= 1;
        }
        assert_eq!(len, 0);
        p
    }
    fn get_items_mut(&mut self) -> &mut [T::Item] {
        Self::FAS_LAYOUT.get_mut(self, self.len)
    }
    pub unsafe fn add_ref(p: *mut Self) {
        (*p).counter += 1;
    }
    pub unsafe fn release(p: *mut Self) {
        let header = &mut *p;
        let c = header.counter;
        if c != 0 {
            header.counter = c - 1;
            return;
        }
        let len = header.len;
        for i in header.get_items_mut() {
            read(i);
        }
        read(&header.value);
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
    use core::alloc::Layout;

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
            let mut i = 0;
            let p = Container::<DebugClean>::alloc(DebugClean(&mut i), [].into_iter());
            assert_eq!(i, 0);
            Container::update::<false>(p);
            assert_eq!(i, 1);
            assert_eq!(counter, 0);
        }
        unsafe {
            counter = 0;
            let mut i = 0;
            let p = Container::<DebugClean>::alloc(
                DebugClean(&mut i),
                [DebugItem(0), DebugItem(1), DebugItem(2)].into_iter(),
            );
            assert_eq!((*p).len, 3);
            Container::update::<true>(p);
            Container::update::<false>(p);
            assert_eq!(i, 0);
            Container::update::<false>(p);
            assert_eq!(i, 1);
            assert_eq!(counter, 3);
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
