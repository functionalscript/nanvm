use std::{
    alloc::{GlobalAlloc, Layout, System},
    ptr::read,
};

#[repr(C)]
pub struct Container<T> {
    counter: usize,
    pub value: T,
    size: usize,
}

pub const DROP: bool = false;
pub const CLONE: bool = true;

impl<T> Container<T> {
    const LAYOUT: Layout = Layout::new::<Container<T>>();
    pub unsafe fn alloc() -> *mut Self {
        System.alloc_zeroed(Self::LAYOUT) as *mut Self
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

    #[test]
    fn test() {
        unsafe {
            let p = Container::<DebugClean>::alloc();
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
            let p = Container::<DebugClean>::alloc();
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
