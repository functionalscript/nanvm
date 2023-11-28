use std::{
    alloc::{GlobalAlloc, System},
    ptr::{drop_in_place, write},
};

use crate::common::fas::FasLayout;

use super::{Base, Info};

#[repr(C)]
pub struct Container<T: Info> {
    base: Base,
    len: usize,
    pub info: T,
}

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
    pub unsafe fn dealloc(p: *mut Self) {
        let container = &mut *p;
        let len = container.len;
        drop_in_place(container.get_items_mut());
        drop_in_place(p);
        System.dealloc(p as *mut u8, Self::FAS_LAYOUT.layout(len));
    }
    pub fn get_items_mut(&mut self) -> &mut [T::Item] {
        Self::FAS_LAYOUT.get_mut(self, self.len)
    }
}

#[cfg(test)]
mod test {
    use core::alloc::Layout;

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::container::Update;

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

    fn add_ref<T: Info>(p: *mut Container<T>) {
        unsafe {
            Base::update(&mut (*p).base, Update::AddRef);
        }
    }

    fn release<T: Info>(p: *mut Container<T>) {
        unsafe {
            if Base::update(&mut (*p).base, Update::Release) != 0 {
                return;
            }
            Container::dealloc(p)
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn sequential_test() {
        unsafe {
            let mut i = 0;
            let p = Container::<DebugClean>::alloc(DebugClean(&mut i), [].into_iter());
            assert_eq!(i, 0);
            release(p);
            assert_eq!(i, 1);
        }
        unsafe {
            let mut item_count = 0;
            let mut clean_count = 0;
            let p = Container::<DebugClean>::alloc(
                DebugClean(&mut clean_count),
                [
                    DebugItem(&mut item_count),
                    DebugItem(&mut item_count),
                    DebugItem(&mut item_count),
                ]
                .into_iter(),
            );
            assert_eq!((*p).len, 3);
            add_ref(p);
            release(p);
            assert_eq!(clean_count, 0);
            assert_eq!(item_count, 0);
            release(p);
            assert_eq!(clean_count, 1);
            assert_eq!(item_count, 3);
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