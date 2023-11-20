use std::alloc::{GlobalAlloc, Layout, System};

#[repr(C)]
pub struct Container<T: Clean> {
    counter: usize,
    value: T,
}

pub trait Clean {
    fn clean(&mut self);
}

impl<T: Clean> Container<T> {
    const LAYOUT: Layout = Layout::new::<Container<T>>();
    pub unsafe fn alloc() -> *mut Self {
        System.alloc_zeroed(Self::LAYOUT) as *mut Self
    }
    pub unsafe fn update<const ADD: bool>(p: *mut Self) {
        let r = &mut *p;
        let c = r.counter;
        if ADD {
            r.counter = c + 1;
        } else {
            if c != 0 {
                r.counter = c - 1;
                return;
            }
            r.value.clean();
            System.dealloc(p as *mut u8, Self::LAYOUT);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct DebugClean(*mut usize);

    impl Clean for DebugClean {
        fn clean(&mut self) {
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
