use std::{
    alloc::{GlobalAlloc, Layout, System},
    mem::{align_of, size_of},
    ptr::read,
};

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

impl<T: Containable> Container<T> {
    const LAYOUT: Layout = Layout::new::<Container<T>>();
    const ITEM_LAYOUT: Layout = Layout::new::<T::Item>();
    pub fn layout(size: usize) -> (Layout, usize) {
        Self::LAYOUT
            .extend(
                Layout::from_size_align(size_of::<T::Item>() * size, align_of::<T::Item>())
                    .unwrap(),
            )
            .unwrap()
    }
    pub unsafe fn alloc(size: usize) -> *mut Self {
        let p = System.alloc_zeroed(Self::layout(size).0) as *mut Self;
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
        System.dealloc(p as *mut u8, Self::LAYOUT);
    }
}

#[cfg(test)]
mod test {
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
        type Item = usize;
    }

    #[test]
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
    fn test2() {
        unsafe {
            let p = Container::<DebugClean>::alloc(0);
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
