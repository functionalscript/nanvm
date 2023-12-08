use core::{
    alloc::Layout,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::{
    atomic_counter::AtomicCounter,
    field_layout::FieldLayout,
    global::{Global, GLOBAL},
    manager::Manager,
};

#[derive(Debug)]
struct Local {
    counter: AtomicUsize,
    size: AtomicUsize,
}

impl Default for Local {
    #[inline(always)]
    fn default() -> Self {
        Self {
            counter: AtomicUsize::new(0),
            size: AtomicUsize::new(0),
        }
    }
}

type Header = *const Local;

type HeaderLayout = FieldLayout<Header, u8>;

impl Local {
    fn layout(block_layout: Layout) -> (HeaderLayout, Layout) {
        let header_layout = HeaderLayout::align_to(block_layout.align());
        let layout = header_layout.layout(block_layout.size());
        (header_layout, layout)
    }
}

impl Manager for &Local {
    type BlockHeader = AtomicCounter;
    unsafe fn alloc(self, block_layout: Layout) -> *mut u8 {
        let (header_layout, layout) = Local::layout(block_layout);
        self.counter.fetch_add(1, Ordering::Relaxed);
        self.size.fetch_add(layout.size(), Ordering::Relaxed);
        let p = GLOBAL.alloc(layout) as *mut Header;
        *p = self;
        header_layout.to_adjacent(&mut *p) as *mut u8
    }
    unsafe fn dealloc(block_p: *mut u8, block_layout: Layout) {
        let (header_layout, layout) = Local::layout(block_layout);
        let p = header_layout.from_adjacent_mut(&mut *block_p);
        {
            let local = &**p;
            local.counter.fetch_sub(1, Ordering::Relaxed);
            local.size.fetch_sub(layout.size(), Ordering::Relaxed);
        }
        Global::dealloc(p as *mut u8, layout);
    }
}

#[cfg(test)]
mod test {
    use core::{mem::size_of, ops::Deref, sync::atomic::Ordering};

    use wasm_bindgen_test::wasm_bindgen_test;

    use crate::mem::{flexible_array::header::FlexibleArrayHeader, manager::Manager};

    use super::Local;

    #[test]
    #[wasm_bindgen_test]
    fn test() {
        const SIZE: usize = size_of::<usize>() + size_of::<isize>() + size_of::<i32>();
        let local = Local::default();
        {
            let mr = local.fixed_new(42);
            assert_eq!(mr.0, 42);
            assert_eq!(local.counter.load(Ordering::Relaxed), 1);
            assert_eq!(local.size.load(Ordering::Relaxed), SIZE);
            let r = mr.to_ref();
            assert_eq!(r.0, 42);
            let mr = r.try_to_mut_ref().unwrap();
            assert_eq!(mr.0, 42);
            let r = mr.to_ref();
            let r1 = r.clone();
            let r2 = r1.try_to_mut_ref().unwrap_err();
            let mr = local.fixed_new(44);
            assert_eq!(mr.0, 44);
            assert_eq!(local.counter.load(Ordering::Relaxed), 2);
            assert_eq!(local.size.load(Ordering::Relaxed), SIZE << 1);
            let mut mr2 = local.flexible_array_new([1, 2, 3].into_iter());
            assert_eq!(mr2.items_mut(), &[1, 2, 3]);
            assert_eq!(local.counter.load(Ordering::Relaxed), 3);
            assert_eq!(
                local.size.load(Ordering::Relaxed),
                (SIZE << 1)
                    + size_of::<usize>()
                    + size_of::<isize>()
                    + size_of::<usize>()
                    + size_of::<[i32; 3]>()
            );
            let r3 = mr2.to_ref();
            assert_eq!(r3.header.len(), 3);
            assert_eq!(r3.items(), &[1, 2, 3]);
            {
                let r4 = r3.clone();
                let r5 = r4.deref();
                // drop(r4);
                assert_eq!(r5.header.len(), 3);
                assert_eq!(r5.items(), &[1, 2, 3]);
                let r6 = &*r4;
                assert_eq!(r5.header.len(), 3);
                assert_eq!(r5.items(), &[1, 2, 3]);
            }
            drop(r2);
            assert_eq!(local.counter.load(Ordering::Relaxed), 3);
            drop(r);
            assert_eq!(local.counter.load(Ordering::Relaxed), 2);
            assert_eq!(
                local.size.load(Ordering::Relaxed),
                SIZE + size_of::<usize>()
                    + size_of::<isize>()
                    + size_of::<usize>()
                    + size_of::<[i32; 3]>()
            );
        }
        assert_eq!(local.counter.load(Ordering::Relaxed), 0);
        assert_eq!(local.size.load(Ordering::Relaxed), 0);
    }
}
