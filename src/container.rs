use std::{
    alloc::{GlobalAlloc, Layout, System},
    mem::{align_of, size_of},
    ptr::read,
};

use crate::const_assert::const_assert;

#[repr(C)]
pub struct Container<T: Containable> {
    counter: usize,
    pub value: T,
    size: usize,
}

pub trait Containable {
    type Item;
}

pub const DROP: bool = false;
pub const CLONE: bool = true;

const fn compatible(t: usize, i: Layout) {
    const_assert(t >= i.align());
    const_assert(t % i.align() == 0);
}

struct ContainableLayout {
    size: usize,
    align: usize,
    item_size: usize,
}

impl ContainableLayout {
    const fn layout(&self, size: usize) -> Layout {
        unsafe {
            Layout::from_size_align_unchecked(
                self.size + self.item_size * size,
                self.align,
            )
        }
    }
}

const fn max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

const fn layout<T: Containable>() -> ContainableLayout {
    let i_align = align_of::<T::Item>();
    let c = Layout::new::<Container<T>>();
    let align = max(c.align(), i_align);
    let size = (c.size() + i_align - 1) / i_align * i_align;
    ContainableLayout {
        align,
        size,
        item_size: size_of::<T::Item>(),
    }
}

impl<T: Containable> Container<T> {
    const LAYOUT: ContainableLayout = layout::<T>();
    pub unsafe fn alloc(size: usize) -> *mut Self {
        let p = System.alloc_zeroed(Self::LAYOUT.layout(size)) as *mut Self;
        (*p).size = size;
        p
    }
    pub unsafe fn update<const ADD: bool>(p: *mut Self) {
        let r = &mut *p;
        let c = r.counter;
        if ADD {
            r.counter = c + 1;
            return;
        }
        if c != 0 {
            r.counter = c - 1;
            return;
        }
        drop(read(&r.value));
        System.dealloc(p as *mut u8, Self::LAYOUT.layout(r.size));
    }
}

#[cfg(test)]
mod test {
    use wasm_bindgen_test::wasm_bindgen_test;

    use super::*;

    struct DebugClean(*mut usize);

    impl Drop for DebugClean {
        fn drop(&mut self) {
            unsafe {
                *self.0 += 1;
            }
        }
    }

    impl Containable for DebugClean {
        type Item = u8;
    }

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        unsafe {
            let p = Container::<DebugClean>::alloc(0);
            let mut i = 0;
            (*p).value.0 = &mut i;
            assert_eq!(i, 0);
            Container::update::<false>(p);
            assert_eq!(i, 1);
        }
    }

    #[test]
    #[wasm_bindgen_test]
    fn test_layout() {
        let cl = Container::<DebugClean>::LAYOUT;
        let x = cl.layout(9);
        let r = Layout::new::<Container<DebugClean>>()
            .extend(Layout::array::<u8>(9).unwrap())
            .unwrap();
        assert_eq!(r.0, x);
        let rt =
            Layout::from_size_align(cl.size + cl.item_size * 9, cl.align).unwrap();
        assert_eq!(x, rt);
    }

    #[test]
    #[wasm_bindgen_test]
    fn test2() {
        unsafe {
            let p = Container::<DebugClean>::alloc(9);
            assert_eq!((*p).size, 9);
            let mut i = 0;
            (*p).value.0 = &mut i;
            Container::update::<true>(p);
            Container::update::<false>(p);
            assert_eq!(i, 0);
            Container::update::<false>(p);
            assert_eq!(i, 1);
        }
    }
}
