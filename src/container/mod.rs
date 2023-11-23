mod base;
mod info;
mod ref_;

use core::{
    alloc::GlobalAlloc,
    ptr::{read, write},
};

use std::alloc::System;

use crate::common::fas::FasLayout;

pub use self::base::{Base, ADD_REF, RELEASE};
pub use self::info::Info;
pub use self::ref_::Ref;

#[repr(C)]
pub struct Container<T: Info> {
    base: Base,
    len: usize,
    pub info: T,
}

pub const DROP: bool = false;
pub const CLONE: bool = true;

impl<T: Info> Container<T> {
    const FAS_LAYOUT: FasLayout<Container<T>, T::Item> = FasLayout::new();
    pub unsafe fn alloc(info: T, items: impl ExactSizeIterator<Item = T::Item>) -> *mut Self {
        let mut len = items.len();
        let p = System.alloc(Self::FAS_LAYOUT.layout(len)) as *mut Self;
        let container = &mut *p;
        write(
            container,
            Container {
                base: Base::default(),
                len,
                info,
            },
        );
        for (dst, src) in container.get_items_mut().iter_mut().zip(items) {
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
        Base::update::<ADD_REF>(&mut (*p).base);
    }
    pub fn dealloc(p: *mut Self) {
        unsafe {
            let container = &mut *p;
            let len = container.len;
            for i in container.get_items_mut() {
                read(i);
            }
            read(&container.info);
            System.dealloc(p as *mut u8, Self::FAS_LAYOUT.layout(len));
        }
    }
    pub unsafe fn release(p: *mut Self) {
        if Base::update::<RELEASE>(&mut (*p).base) != 0 {
            return;
        }
        Self::dealloc(p)
    }
    #[inline(always)]
    pub unsafe fn update<const I: isize>(p: *mut Self) {
        if I == 1 {
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

    struct DebugItem(*mut usize);

    impl Drop for DebugClean {
        fn drop(&mut self) {
            unsafe {
                *self.0 += 1;
            }
        }
    }

    impl Drop for DebugItem {
        fn drop(&mut self) {
            unsafe {
                *self.0 += 1;
            }
        }
    }

    impl Info for DebugClean {
        type Item = DebugItem;
    }

    #[test]
    #[wasm_bindgen_test]
    fn sequential_test() {
        unsafe {
            let mut i = 0;
            let p = Container::<DebugClean>::alloc(DebugClean(&mut i), [].into_iter());
            assert_eq!(i, 0);
            Container::update::<-1>(p);
            assert_eq!(i, 1);
        }
        unsafe {
            let mut counter = 0;
            let mut i = 0;
            let p = Container::<DebugClean>::alloc(
                DebugClean(&mut i),
                [
                    DebugItem(&mut counter),
                    DebugItem(&mut counter),
                    DebugItem(&mut counter),
                ]
                .into_iter(),
            );
            assert_eq!((*p).len, 3);
            Container::update::<1>(p);
            Container::update::<-1>(p);
            assert_eq!(i, 0);
            Container::update::<-1>(p);
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
            .extend(Layout::array::<DebugItem>(9).unwrap())
            .unwrap();
        assert_eq!(r.0, x);
    }
}
